use anyhow::{Result, anyhow};
use log::{debug, info};
use serde_json::Value;

use lsp_types::{
    ClientCapabilities, CompletionParams, DidOpenTextDocumentParams, DocumentSymbolParams,
    InitializeParams, InitializeResult, InitializedParams, Position,
    TextDocumentClientCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, WindowClientCapabilities, lsp_notification, lsp_request,
    notification::Notification,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout},
};

use crate::clang::lsp_server_wraper;

pub struct ClangdClient {
    server_process: tokio::process::Child,
    request_id: i64,
}

impl ClangdClient {
    pub async fn warpper_loop(
        mut self,
        request_rx: tokio::sync::mpsc::UnboundedReceiver<u32>,
        response_tx: tokio::sync::mpsc::UnboundedSender<u32>,
    ) -> Result<()> {
        let mut stdin = self
            .server_process
            .stdin
            .take()
            .expect("Failed to open stdin");
        let stdout = self
            .server_process
            .stdout
            .take()
            .expect("Failed to open stdin");

        stdin = self.initialize(stdin).await?;

        tokio::spawn(Self::loop_response(stdout, response_tx));
        tokio::spawn(self.loop_request(stdin, request_rx));
        Ok(())
    }

    /// Start a new clangd server process and initialize the LSP connection
    pub fn new(log_path: &String, compile_commands_dir: String, debug: bool) -> Result<Self> {
        let log_file = std::fs::File::create(log_path)?;
        let log_level = match debug {
            true => "verbose",
            false => "info",
        };
        let jobs = std::thread::available_parallelism()
            .map(|n| n.get().saturating_mul(2))
            .unwrap_or(2)
            .to_string();
        // Start clangd process
        let child = tokio::process::Command::new("clangd")
            .arg(format!("--compile-commands-dir={}", compile_commands_dir))
            .arg(format!("--log={}", log_level))
            .arg("--background-index")
            .arg("--pch-storage=memory")
            .arg("--background-index-priority=normal")
            .arg("-j")
            .arg(&jobs)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(log_file)
            .spawn()?;

        let process_id = child
            .id()
            .map_or(Err(anyhow!("lsp-wrapper start fail.")), |id| Ok(id))?;
        info!("clangd server started with PID: {}", process_id);

        let client = ClangdClient {
            server_process: child,
            request_id: 0,
        };

        Ok(client)
    }
}

impl ClangdClient {
    async fn request<P: serde::Serialize>(
        &mut self,
        mut stdin: ChildStdin,
        method: &str,
        params: P,
    ) -> Result<ChildStdin> {
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
        let str = format!("Content-Length: {}\r\n\r\n{}", content_length, request_str);

        stdin.write_all(str.as_bytes()).await?;
        stdin.flush().await?;
        Ok(stdin)
    }

    async fn notification<P: serde::Serialize>(
        &mut self,
        mut stdin: ChildStdin,
        method: &str,
        params: P,
    ) -> Result<ChildStdin> {
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

        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(notification_str.as_bytes()).await?;
        stdin.flush().await?;

        Ok(stdin)
    }

    async fn response(
        mut reader: BufReader<ChildStdout>,
    ) -> Result<(Value, BufReader<ChildStdout>)> {
        // Read headers
        let mut content_length = None;
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).await?;
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
        reader.read_exact(&mut buffer).await?;

        // Parse the response
        let response: serde_json::Value = serde_json::from_slice(&buffer)?;
        // Check for errors
        if let Some(error) = response.get("error") {
            return Err(anyhow!("LSP error: {}", error));
        }
        return Ok((response, reader));
    }

    async fn loop_request(
        mut self,
        stdin: ChildStdin,
        request_rx: tokio::sync::mpsc::UnboundedReceiver<u32>,
    ) -> () {
        loop {}
    }

    async fn loop_response(
        stdout: ChildStdout,
        response_tx: tokio::sync::mpsc::UnboundedSender<u32>,
    ) -> () {
        let mut reader = BufReader::new(stdout);
        loop {
            let (value, reader_new) = Self::response(reader).await.unwrap();
            reader = reader_new;
            debug!("respose {:?}", value);
        }
    }

    /// Send the initialize request to the server
    async fn initialize(&mut self, stdin: ChildStdin) -> Result<ChildStdin> {
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
                    augments_syntax_tokens: Some(true),
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
                        lsp_types::SemanticTokenType::ENUM,
                        lsp_types::SemanticTokenType::INTERFACE,
                        lsp_types::SemanticTokenType::STRUCT,
                        lsp_types::SemanticTokenType::TYPE_PARAMETER,
                        lsp_types::SemanticTokenType::PARAMETER,
                        lsp_types::SemanticTokenType::VARIABLE,
                        lsp_types::SemanticTokenType::PROPERTY,
                        lsp_types::SemanticTokenType::ENUM_MEMBER,
                        lsp_types::SemanticTokenType::EVENT,
                        lsp_types::SemanticTokenType::FUNCTION,
                        lsp_types::SemanticTokenType::METHOD,
                        lsp_types::SemanticTokenType::MACRO,
                        lsp_types::SemanticTokenType::KEYWORD,
                        lsp_types::SemanticTokenType::MODIFIER,
                        lsp_types::SemanticTokenType::COMMENT,
                        lsp_types::SemanticTokenType::STRING,
                        lsp_types::SemanticTokenType::NUMBER,
                        lsp_types::SemanticTokenType::REGEXP,
                        lsp_types::SemanticTokenType::OPERATOR,
                        lsp_types::SemanticTokenType::DECORATOR,
                    ],
                    token_modifiers: vec![
                        lsp_types::SemanticTokenModifier::DECLARATION,
                        lsp_types::SemanticTokenModifier::DEFINITION,
                        lsp_types::SemanticTokenModifier::READONLY,
                        lsp_types::SemanticTokenModifier::STATIC,
                        lsp_types::SemanticTokenModifier::DEPRECATED,
                        lsp_types::SemanticTokenModifier::ABSTRACT,
                        lsp_types::SemanticTokenModifier::ASYNC,
                        lsp_types::SemanticTokenModifier::MODIFICATION,
                        lsp_types::SemanticTokenModifier::DOCUMENTATION,
                        lsp_types::SemanticTokenModifier::DEFAULT_LIBRARY,
                    ],
                    formats: vec![lsp_types::TokenFormat::RELATIVE],
                    overlapping_token_support: Some(false),
                    multiline_token_support: Some(false),

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
        let mut stdin = stdin;
        // Send initialize request
        stdin = self.request(stdin, "initialize", params).await?;

        // Send initialized notification
        stdin = self
            .notification(stdin, "initialized", InitializedParams {})
            .await?;

        Ok(stdin)
    }
}
