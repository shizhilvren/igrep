use anyhow::{anyhow, Result};
use futures::StreamExt;
use jsonrpsee::{
    core::{
        client::{ClientT, Subscription, SubscriptionClientT},
        params::ObjectParams,
        Error as JsonRpcError,
    },
    proc_macros::rpc,
    rpc_params,
    types::error::ErrorObjectOwned,
};
use lsp_types::{
    ClientCapabilities, CompletionParams, DidOpenTextDocumentParams, InitializeParams, 
    InitializeResult, Position, TextDocumentClientCapabilities, TextDocumentIdentifier, 
    TextDocumentItem, TextDocumentPositionParams, Url, InitializedParams,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Child, Command, Stdio},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

// Define LSP JSON-RPC interface
#[rpc(client)]
pub trait LspClient {
    #[method(name = "initialize")]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, JsonRpcError>;
    
    #[method(name = "textDocument/hover")]
    async fn hover(&self, params: TextDocumentPositionParams) -> Result<Value, JsonRpcError>;
    
    #[method(name = "textDocument/definition")]
    async fn goto_definition(&self, params: TextDocumentPositionParams) -> Result<Value, JsonRpcError>;
    
    #[method(name = "textDocument/completion")]
    async fn completion(&self, params: CompletionParams) -> Result<Value, JsonRpcError>;
    
    // Notifications
    #[method(name = "initialized")]
    fn initialized(&self, params: InitializedParams) -> Result<(), JsonRpcError>;
    
    #[method(name = "textDocument/didOpen")]
    fn did_open_text_document(&self, params: DidOpenTextDocumentParams) -> Result<(), JsonRpcError>;
    
    #[method(name = "exit")]
    fn exit(&self) -> Result<(), JsonRpcError>;
}

/// A simple LSP client for interacting with clangd, using jsonrpsee
pub struct ClangdClient {
    server_process: Child,
    client: LspClientClient,
    runtime: tokio::runtime::Runtime,
}

impl ClangdClient {
    /// Start a new clangd server process and initialize the LSP connection
    pub fn new() -> Result<Self> {
        // Create a tokio runtime for async operations
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        
        // Start clangd process
        let mut child = Command::new("clangd")
            .arg("--log=verbose")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        // Get process stdin/stdout
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to get stdout"))?;
        
        // Create the jsonrpsee client inside the runtime
        let client = runtime.block_on(async {
            // Create channels for communication
            let (tx_stdout, mut rx_stdout) = mpsc::channel::<String>(100);
            let (tx_send, rx_send) = mpsc::channel::<String>(100);
            
            // Setup stdout reading task
            let mut buf_reader = BufReader::new(stdout).lines();
            tokio::spawn(async move {
                let mut content_length: Option<usize> = None;
                
                while let Some(Ok(line)) = buf_reader.next_line().await {
                    if line.starts_with("Content-Length: ") {
                        content_length = Some(
                            line["Content-Length: ".len()..].trim().parse().unwrap_or(0)
                        );
                    } else if line.is_empty() && content_length.is_some() {
                        // Read the message body
                        let mut buffer = vec![0; content_length.unwrap()];
                        // Read exact number of bytes for the JSON-RPC message
                        // This is simplified - in a real implementation you'd need to properly read the exact bytes
                        if let Some(Ok(msg)) = buf_reader.next_line().await {
                            if !msg.is_empty() {
                                let _ = tx_stdout.send(msg).await;
                            }
                        }
                        content_length = None;
                    }
                }
            });
            
            // Setup stdin writing task
            let mut writer = stdin;
            tokio::spawn(async move {
                while let Some(message) = rx_send.recv().await {
                    let header = format!("Content-Length: {}\r\n\r\n", message.len());
                    if writer.write_all(header.as_bytes()).is_err() ||
                       writer.write_all(message.as_bytes()).is_err() ||
                       writer.flush().is_err() {
                        break;
                    }
                }
            });
            
            // Create custom JSON-RPC client
            let client = LspClientClient::new(
                Box::new(move |method, params| {
                    let tx = tx_send.clone();
                    Box::pin(async move {
                        let request = json!({
                            "jsonrpc": "2.0",
                            "id": 1, // In real implementation, generate unique IDs
                            "method": method,
                            "params": params
                        });
                        
                        let request_str = serde_json::to_string(&request)
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                        
                        tx.send(request_str).await
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                        
                        // Wait for response from stdout
                        if let Some(response_str) = rx_stdout.recv().await {
                            let response: Value = serde_json::from_str(&response_str)
                                .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                            
                            if let Some(error) = response.get("error") {
                                let error_obj = serde_json::from_value::<ErrorObjectOwned>(error.clone())
                                    .unwrap_or_else(|_| ErrorObjectOwned::new(0, error.to_string(), None));
                                return Err(JsonRpcError::Call(error_obj));
                            }
                            
                            let result = response.get("result").cloned().unwrap_or(Value::Null);
                            Ok(result)
                        } else {
                            Err(JsonRpcError::Custom("No response received".to_string()))
                        }
                    })
                }),
                Box::new(move |method, params| {
                    let tx = tx_send.clone();
                    Box::pin(async move {
                        let notification = json!({
                            "jsonrpc": "2.0",
                            "method": method,
                            "params": params
                        });
                        
                        let notification_str = serde_json::to_string(&notification)
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                        
                        tx.send(notification_str).await
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                            
                        Ok(())
                    })
                }),
                Box::new(|_method, _params| {
                    Box::pin(async { 
                        Err(JsonRpcError::Custom("Subscriptions not supported".to_string())) 
                    })
                })
            );
            
            client
        });
        
        // Create the client
        let mut lsp_client = ClangdClient {
            server_process: child,
            client,
            runtime,
        };
        
        // Initialize the LSP connection
        lsp_client.initialize()?;
        
        Ok(lsp_client)
    }
    
    /// Start clangd with specific compile commands directory
    pub fn with_compile_commands(compile_commands_dir: &str) -> Result<Self> {
        // Create a tokio runtime for async operations
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
            
        // Start clangd process with compilation database
        let mut child = Command::new("clangd")
            .arg("--log=verbose")
            .arg(format!("--compile-commands-dir={}", compile_commands_dir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        // The implementation here is the same as in new(), except for the clangd command
        // Get process stdin/stdout
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to get stdout"))?;
        
        // Create the jsonrpsee client inside the runtime
        let client = runtime.block_on(async {
            // Create channels for communication
            let (tx_stdout, mut rx_stdout) = mpsc::channel::<String>(100);
            let (tx_send, rx_send) = mpsc::channel::<String>(100);
            
            // Setup stdout reading task
            let mut buf_reader = BufReader::new(stdout).lines();
            tokio::spawn(async move {
                let mut content_length: Option<usize> = None;
                
                while let Some(Ok(line)) = buf_reader.next_line().await {
                    if line.starts_with("Content-Length: ") {
                        content_length = Some(
                            line["Content-Length: ".len()..].trim().parse().unwrap_or(0)
                        );
                    } else if line.is_empty() && content_length.is_some() {
                        // Read the message body
                        let mut buffer = vec![0; content_length.unwrap()];
                        // Read exact number of bytes for the JSON-RPC message
                        // This is simplified - in a real implementation you'd need to properly read the exact bytes
                        if let Some(Ok(msg)) = buf_reader.next_line().await {
                            if !msg.is_empty() {
                                let _ = tx_stdout.send(msg).await;
                            }
                        }
                        content_length = None;
                    }
                }
            });
            
            // Setup stdin writing task
            let mut writer = stdin;
            tokio::spawn(async move {
                while let Some(message) = rx_send.recv().await {
                    let header = format!("Content-Length: {}\r\n\r\n", message.len());
                    if writer.write_all(header.as_bytes()).is_err() ||
                       writer.write_all(message.as_bytes()).is_err() ||
                       writer.flush().is_err() {
                        break;
                    }
                }
            });
            
            // Create custom JSON-RPC client
            let client = LspClientClient::new(
                Box::new(move |method, params| {
                    let tx = tx_send.clone();
                    Box::pin(async move {
                        let request = json!({
                            "jsonrpc": "2.0",
                            "id": 1, // In real implementation, generate unique IDs
                            "method": method,
                            "params": params
                        });
                        
                        let request_str = serde_json::to_string(&request)
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                        
                        tx.send(request_str).await
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                        
                        // Wait for response from stdout
                        if let Some(response_str) = rx_stdout.recv().await {
                            let response: Value = serde_json::from_str(&response_str)
                                .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                            
                            if let Some(error) = response.get("error") {
                                let error_obj = serde_json::from_value::<ErrorObjectOwned>(error.clone())
                                    .unwrap_or_else(|_| ErrorObjectOwned::new(0, error.to_string(), None));
                                return Err(JsonRpcError::Call(error_obj));
                            }
                            
                            let result = response.get("result").cloned().unwrap_or(Value::Null);
                            Ok(result)
                        } else {
                            Err(JsonRpcError::Custom("No response received".to_string()))
                        }
                    })
                }),
                Box::new(move |method, params| {
                    let tx = tx_send.clone();
                    Box::pin(async move {
                        let notification = json!({
                            "jsonrpc": "2.0",
                            "method": method,
                            "params": params
                        });
                        
                        let notification_str = serde_json::to_string(&notification)
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                        
                        tx.send(notification_str).await
                            .map_err(|e| JsonRpcError::Custom(e.to_string()))?;
                            
                        Ok(())
                    })
                }),
                Box::new(|_method, _params| {
                    Box::pin(async { 
                        Err(JsonRpcError::Custom("Subscriptions not supported".to_string())) 
                    })
                })
            );
            
            client
        });
        
        // Create the client
        let mut lsp_client = ClangdClient {
            server_process: child,
            client,
            runtime,
        };
        
        // Initialize the LSP connection
        lsp_client.initialize()?;
        
        Ok(lsp_client)
    }

    /// Send the initialize request to the server
    fn initialize(&mut self) -> Result<InitializeResult> {
        // Prepare initialize parameters
        let params = InitializeParams {
            process_id: Some(std::process::id() as i64),
            root_uri: None,  // We're not opening a project
            client_info: Some(lsp_types::ClientInfo {
                name: "igrep-clangd-client".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    completion: Some(lsp_types::CompletionClientCapabilities::default()),
                    hover: Some(lsp_types::HoverClientCapabilities::default()),
                    definition: Some(lsp_types::GotoCapability::default()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            initialization_options: None,
            trace: Some(lsp_types::TraceValue::Verbose),
            workspace_folders: None,
            ..Default::default()
        };

        // Send initialize request using jsonrpsee
        let result = self.runtime.block_on(async {
            self.client.initialize(params)
                .await
                .map_err(|e| anyhow!("LSP initialize error: {}", e))
        })?;
        
        // Send initialized notification
        self.runtime.block_on(async {
            self.client.initialized(InitializedParams {})
                .map_err(|e| anyhow!("LSP initialized error: {}", e))
        })?;
        
        Ok(result)
    }

    /// Open a file in the LSP server
    pub fn open_file(&mut self, file_path: &str, content: String) -> Result<()> {
        let uri = Url::parse(&format!("file://{}", file_path))?;
        
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "cpp".to_string(),  // or "c" depending on file type
                version: 1,
                text: content,
            },
        };

        self.runtime.block_on(async {
            self.client.did_open_text_document(params)
                .map_err(|e| anyhow!("LSP didOpen error: {}", e))
        })
    }

    /// Get code completion suggestions at a position in a file
    pub fn get_completions(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Url::parse(&format!("file://{}", file_path))?;
        
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        self.runtime.block_on(async {
            self.client.completion(params)
                .map_err(|e| anyhow!("LSP completion error: {}", e))
        })
    }

    /// Go to definition of symbol at position
    pub fn goto_definition(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Url::parse(&format!("file://{}", file_path))?;
        
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        self.runtime.block_on(async {
            self.client.goto_definition(params)
                .map_err(|e| anyhow!("LSP definition error: {}", e))
        })
    }
    
    /// Get hover information at a position in a file
    pub fn get_hover(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Url::parse(&format!("file://{}", file_path))?;
        
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        self.runtime.block_on(async {
            self.client.hover(params)
                .map_err(|e| anyhow!("LSP hover error: {}", e))
        })
    }
}

impl Drop for ClangdClient {
    fn drop(&mut self) {
        // Try to gracefully shut down the server
        let _ = self.runtime.block_on(async {
            self.client.exit()
        });
        let _ = self.server_process.kill();
    }
}
