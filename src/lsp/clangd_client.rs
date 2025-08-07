use anyhow::{Result, anyhow};
use futures::io::Window;
use lsp_types::{
    ClientCapabilities, CompletionParams, DidOpenTextDocumentParams, DocumentSymbolParams,
    InitializeParams, InitializeResult, InitializedParams, Position,
    TextDocumentClientCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, WindowClientCapabilities,
};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    str::FromStr,
};

/// A simple LSP client for interacting with clangd
pub struct ClangdClient {
    server_process: Child,
    request_id: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ASTParams {
    text_document: TextDocumentIdentifier,
    range: lsp_types::Range,
}

impl ClangdClient {
    /// Start a new clangd server process and initialize the LSP connection
    pub fn new(log_path: &String, compile_commands_dir: &str, debug: bool) -> Result<Self> {
        let log_file = std::fs::File::create(log_path)?;
        let log_level = match debug {
            true => "verbose",
            false => "info",
        };
        // Start clangd process
        let child = Command::new("clangd")
            .arg(format!("--compile-commands-dir={}", compile_commands_dir))
            .arg(format!("--log={}", log_level))
            .arg("--background-index")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(log_file)
            .spawn()?;

        println!("clangd server started with PID: {}", child.id());

        let mut client = ClangdClient {
            server_process: child,
            request_id: 0,
        };

        // Initialize the LSP connection
        client.initialize()?;

        Ok(client)
    }

    /// Send the initialize request to the server
    fn initialize(&mut self) -> Result<InitializeResult> {
        // 创建详细的客户端能力
        let client_capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                // 补全功能
                completion: None,

                // 悬停功能
                hover: Some(lsp_types::HoverClientCapabilities {
                    dynamic_registration: Some(true),
                    content_format: Some(vec![
                        lsp_types::MarkupKind::Markdown,
                        lsp_types::MarkupKind::PlainText,
                    ]),
                }),

                // 定义跳转功能
                definition: Some(lsp_types::GotoCapability {
                    dynamic_registration: Some(true),
                    link_support: Some(true),
                }),

                // 符号查询功能
                document_symbol: Some(lsp_types::DocumentSymbolClientCapabilities {
                    dynamic_registration: Some(true),
                    symbol_kind: Some(lsp_types::SymbolKindCapability {
                        value_set: Some(vec![
                            lsp_types::SymbolKind::FILE,
                            lsp_types::SymbolKind::MODULE,
                            lsp_types::SymbolKind::NAMESPACE,
                            lsp_types::SymbolKind::PACKAGE,
                            lsp_types::SymbolKind::CLASS,
                            lsp_types::SymbolKind::METHOD,
                            lsp_types::SymbolKind::PROPERTY,
                            lsp_types::SymbolKind::FIELD,
                            lsp_types::SymbolKind::CONSTRUCTOR,
                            lsp_types::SymbolKind::ENUM,
                            lsp_types::SymbolKind::INTERFACE,
                            lsp_types::SymbolKind::FUNCTION,
                            lsp_types::SymbolKind::VARIABLE,
                            lsp_types::SymbolKind::CONSTANT,
                            lsp_types::SymbolKind::STRING,
                            lsp_types::SymbolKind::NUMBER,
                            lsp_types::SymbolKind::BOOLEAN,
                            lsp_types::SymbolKind::ARRAY,
                            lsp_types::SymbolKind::OBJECT,
                            lsp_types::SymbolKind::KEY,
                            lsp_types::SymbolKind::NULL,
                            lsp_types::SymbolKind::ENUM_MEMBER,
                            lsp_types::SymbolKind::STRUCT,
                            lsp_types::SymbolKind::EVENT,
                            lsp_types::SymbolKind::OPERATOR,
                            lsp_types::SymbolKind::TYPE_PARAMETER,
                        ]),
                    }),
                    hierarchical_document_symbol_support: Some(true),
                    tag_support: Some(lsp_types::TagSupport {
                        value_set: vec![lsp_types::SymbolTag::DEPRECATED],
                    }),
                    ..Default::default()
                }),

                // 诊断功能
                publish_diagnostics: None,

                // 语义标记功能
                semantic_tokens: Some(lsp_types::SemanticTokensClientCapabilities {
                    dynamic_registration: Some(true),
                    requests: lsp_types::SemanticTokensClientCapabilitiesRequests {
                        range: None,
                        full: Some(lsp_types::SemanticTokensFullOptions::Bool(true)),
                        ..Default::default()
                    },
                    token_types: vec![
                        lsp_types::SemanticTokenType::NAMESPACE,
                        lsp_types::SemanticTokenType::TYPE,
                        lsp_types::SemanticTokenType::CLASS,
                        lsp_types::SemanticTokenType::INTERFACE,
                        lsp_types::SemanticTokenType::STRUCT,
                        lsp_types::SemanticTokenType::TYPE_PARAMETER,
                        lsp_types::SemanticTokenType::PARAMETER,
                        lsp_types::SemanticTokenType::VARIABLE,
                        lsp_types::SemanticTokenType::PROPERTY,
                        lsp_types::SemanticTokenType::ENUM,
                        lsp_types::SemanticTokenType::ENUM_MEMBER,
                        lsp_types::SemanticTokenType::EVENT,
                        lsp_types::SemanticTokenType::FUNCTION,
                        lsp_types::SemanticTokenType::METHOD,
                        lsp_types::SemanticTokenType::KEYWORD,
                        lsp_types::SemanticTokenType::MODIFIER,
                        lsp_types::SemanticTokenType::COMMENT,
                        lsp_types::SemanticTokenType::STRING,
                        lsp_types::SemanticTokenType::NUMBER,
                        lsp_types::SemanticTokenType::REGEXP,
                        lsp_types::SemanticTokenType::OPERATOR,
                    ],
                    token_modifiers: vec![
                        lsp_types::SemanticTokenModifier::DECLARATION,
                        lsp_types::SemanticTokenModifier::DEFINITION,
                        lsp_types::SemanticTokenModifier::READONLY,
                        lsp_types::SemanticTokenModifier::STATIC,
                        lsp_types::SemanticTokenModifier::DEPRECATED,
                        lsp_types::SemanticTokenModifier::ASYNC,
                        lsp_types::SemanticTokenModifier::MODIFICATION,
                        lsp_types::SemanticTokenModifier::DOCUMENTATION,
                        lsp_types::SemanticTokenModifier::DEFAULT_LIBRARY,
                    ],
                    formats: vec![lsp_types::TokenFormat::RELATIVE],
                    overlapping_token_support: Some(true),
                    multiline_token_support: Some(true),

                    ..Default::default()
                }),

                code_lens: Some(lsp_types::CodeLensClientCapabilities {
                    dynamic_registration: Some(true),
                    ..Default::default()
                }),

                // 添加更多需要的功能...
                ..Default::default()
            }),

            // 窗口功能
            window: Some(lsp_types::WindowClientCapabilities {
                work_done_progress: Some(true),
                ..Default::default()
            }),

            ..Default::default()
        };

        // Prepare initialize parameters
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            client_info: Some(lsp_types::ClientInfo {
                name: "igrep-clangd-client".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: client_capabilities,
            initialization_options: None,
            trace: Some(lsp_types::TraceValue::Verbose),
            workspace_folders: None,
            ..Default::default()
        };

        // Send initialize request
        let result: InitializeResult = self.send_request("initialize", params)?;

        // Send initialized notification
        self.send_notification("initialized", InitializedParams {})?;

        Ok(result)
    }

    /// Open a file in the LSP server
    pub fn open_file(&mut self, file_path: &str, content: String) -> Result<()> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "cpp".to_string(), // or "c" depending on file type
                version: 1,
                text: content,
            },
        };

        self.send_notification("textDocument/didOpen", params)
    }

    pub fn get_code_lens(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        self.send_request("textDocument/codeLens", params)
    }

    pub fn get_ast(&mut self, file_path: &str) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = ASTParams {
            text_document: TextDocumentIdentifier { uri },
            range: lsp_types::Range {
                start: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: 1,
                },
            },
        };

        self.send_request("textDocument/ast", params)
    }

    /// Get code completion suggestions at a position in a file
    pub fn get_completions(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        self.send_request("textDocument/completion", params)
    }

    /// Go to definition of symbol at position
    pub fn goto_definition(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        self.send_request("textDocument/definition", params)
    }

    /// Get hover information at a position in a file
    pub fn get_hover(&mut self, file_path: &str, line: u32, character: u32) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        self.send_request("textDocument/hover", params)
    }

    pub fn get_semantic_tokens_full(&mut self, file_path: &str) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = lsp_types::SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request("textDocument/semanticTokens/full", params)
    }

    pub fn get_symbols(&mut self, file_path: &str) -> Result<Value> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request("textDocument/documentSymbol", params)
    }
    /// Send a request to the LSP server and parse the response
    fn send_request<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R> {
        self.request_id += 1;
        let id = self.request_id;

        // Create request JSON
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        // Serialize and send the request
        let request_str = serde_json::to_string(&request)?;
        let content_length = request_str.len();
        let header = format!("Content-Length: {}\r\n\r\n", content_length);

        let stdin = self
            .server_process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;
        stdin.write_all(header.as_bytes())?;
        stdin.write_all(request_str.as_bytes())?;
        stdin.flush()?;

        // Read and parse the response
        let stdout = self
            .server_process
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdout"))?;
        let mut reader = BufReader::new(stdout);

        // Read headers
        let mut content_length = None;
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            if line == "\r\n" || line.is_empty() {
                break;
            }
            if line.starts_with("Content-Length: ") {
                content_length = Some(line["Content-Length: ".len()..].trim().parse::<usize>()?);
            }
        }

        // Read the response body
        let content_length = content_length.ok_or_else(|| anyhow!("No Content-Length header"))?;
        let mut buffer = vec![0; content_length];
        reader.read_exact(&mut buffer)?;

        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&buffer)?;

        // Check for errors
        if let Some(error) = response.get("error") {
            return Err(anyhow!("LSP error: {}", error));
        }

        // Extract and deserialize result
        let result = response["result"].clone();
        Ok(serde_json::from_value(result)?)
    }

    /// Send a notification to the LSP server (no response expected)
    fn send_notification<P: serde::Serialize>(&mut self, method: &str, params: P) -> Result<()> {
        // Create notification JSON
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        // Serialize and send the notification
        let notification_str = serde_json::to_string(&notification)?;
        let content_length = notification_str.len();
        let header = format!("Content-Length: {}\r\n\r\n", content_length);

        let stdin = self
            .server_process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;
        stdin.write_all(header.as_bytes())?;
        stdin.write_all(notification_str.as_bytes())?;
        stdin.flush()?;

        Ok(())
    }
}

impl Drop for ClangdClient {
    fn drop(&mut self) {
        // Try to gracefully shut down the server
        let _ = self.send_notification("exit", serde_json::Value::Null);
        let _ = self.server_process.kill();
    }
}
