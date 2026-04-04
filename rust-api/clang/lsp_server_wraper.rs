use anyhow::{Result, anyhow};
use log::{debug, error, info};
use serde::{Serialize, Serializer};
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
    sync::{mpsc, oneshot},
};

use crate::clang::lsp_server_wraper;

struct ResponseToClientData {
    pub val: Value,
}
struct ClientToRequestData {
    sender: tokio::sync::oneshot::Sender<ResponseToClientData>,
    method: String,
    params: Value,
}
struct RequestToResponseData {
    id: RequestID,
    response_tx: tokio::sync::oneshot::Sender<ResponseToClientData>,
}

#[derive(Clone, Serialize, PartialEq, Eq)]
struct RequestID {
    id: u64,
}
#[derive(Clone)]
pub struct RequestClient {
    request_tx: mpsc::Sender<ClientToRequestData>,
}

impl RequestClient {
    pub fn rqueset<P: serde::Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<tokio::sync::oneshot::Receiver<ResponseToClientData>> {
        let (response_to_client_tx, response_to_client_rx) =
            oneshot::channel::<ResponseToClientData>();
        let valus = serde_json::json!(params);
        let data = ClientToRequestData::from((method, valus, response_to_client_tx));
        self.request_tx.send(data);
        Ok(response_to_client_rx)
    }
}

pub struct ClangdClient {
    server_process: tokio::process::Child,
    request_id: RequestID,
}

impl ClangdClient {
    pub async fn warpper_loop(
        mut self,
        request_rx: tokio::sync::mpsc::UnboundedReceiver<ClientToRequestData>,
        // response_tx: tokio::sync::mpsc::UnboundedSender<u32>,
    ) -> Result<RequestClient> {
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

        let (req_to_res_tx, req_to_res_rx) =
            tokio::sync::mpsc::unbounded_channel::<RequestToResponseData>();

        tokio::spawn(Self::loop_response(stdout, req_to_res_rx));
        tokio::spawn(self.loop_request(stdin, request_rx, req_to_res_tx));
        Err(anyhow!("error"))
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
    async fn request_base<P: serde::Serialize>(
        &mut self,
        mut stdin: ChildStdin,
        use_id: bool,
        method: &str,
        params: P,
    ) -> Result<(ChildStdin, Option<RequestID>)> {
        let (request, id) = match use_id {
            true => {
                self.request_id.next();
                let id = self.request_id.clone();
                // Create request JSON
                (
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id.clone(),
                        "method": method,
                        "params": params
                    }),
                    Some(id),
                )
            }
            false => (
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params
                }),
                None,
            ),
        };

        // Serialize and send the request
        let request_str = serde_json::to_string(&request)?;
        let content_length = request_str.len();
        let str = format!("Content-Length: {}\r\n\r\n{}", content_length, request_str);

        stdin.write_all(str.as_bytes()).await?;
        stdin.flush().await?;
        Ok((stdin, id))
    }

    async fn request<P: serde::Serialize>(
        &mut self,
        stdin: ChildStdin,
        method: &str,
        params: P,
    ) -> Result<(ChildStdin, RequestID)> {
        let (stdin, id) = self.request_base(stdin, true, method, params).await?;
        id.map_or(Err(anyhow!("request must have id")), |id| Ok((stdin, id)))
    }

    async fn notification<P: serde::Serialize>(
        &mut self,
        mut stdin: ChildStdin,
        method: &str,
        params: P,
    ) -> Result<ChildStdin> {
        let (stdin, _) = self.request_base(stdin, false, method, params).await?;
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
        client_to_request_rx: tokio::sync::mpsc::UnboundedReceiver<ClientToRequestData>,
        request_to_response_tx: tokio::sync::mpsc::UnboundedSender<RequestToResponseData>,
    ) -> () {
        let mut stdin = stdin;
        let mut client_to_request_rx = client_to_request_rx;
        loop {
            match client_to_request_rx.recv().await {
                Some(data) => match self.request(stdin, &data.method, data.params).await {
                    Ok((stdin_new, id)) => {
                        stdin = stdin_new;
                        match request_to_response_tx
                            .send(RequestToResponseData::from((id, data.sender)))
                        {
                            Err(e) => {
                                error!("request fail {}", e);
                                break;
                            }
                            Ok(_) => {}
                        }
                    }
                    Err(e) => {
                        error!("request fail {} for {}", e, &data.method);
                        break;
                    }
                },
                None => {
                    error!("loop request finish because client_to_request close");
                    break;
                }
            }
        }
    }

    async fn loop_response(
        stdout: ChildStdout,
        req_to_res_rx: tokio::sync::mpsc::UnboundedReceiver<RequestToResponseData>,
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

impl RequestID {
    pub fn next(&mut self) {
        self.id += 1
    }
}

impl From<()> for RequestID {
    fn from(value: ()) -> Self {
        RequestID { id: 0 }
    }
}
impl From<(&str, Value, oneshot::Sender<ResponseToClientData>)> for ClientToRequestData {
    fn from(
        (method, params, sender): (&str, Value, oneshot::Sender<ResponseToClientData>),
    ) -> Self {
        ClientToRequestData {
            sender,
            method: method.to_string(),
            params,
        }
    }
}

impl From<(RequestID, oneshot::Sender<ResponseToClientData>)> for RequestToResponseData {
    fn from((id, sender): (RequestID, oneshot::Sender<ResponseToClientData>)) -> Self {
        RequestToResponseData {
            id,
            response_tx: sender,
        }
    }
}
