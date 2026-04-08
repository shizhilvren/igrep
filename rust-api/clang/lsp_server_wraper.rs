use std::{collections::HashMap, str::FromStr};

use anyhow::{Result, anyhow};
use log::{debug, error, info, warn};
use serde_json::Value;

use lsp_types::{
    ClientCapabilities, DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams,
    Position, TextDocumentClientCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout},
    sync::{mpsc, oneshot},
};

#[derive(Debug)]
pub struct ResponseToClientData {
    pub val: Value,
}

struct ClientToRequestData {
    sender: ClientToRequestDataType,
    method: String,
    params: Value,
}
enum ClientToRequestDataType {
    Request(tokio::sync::oneshot::Sender<ResponseToClientData>),
    Notification,
    Response(RequestID),
}

#[derive(Debug)]
struct RequestToResponseData {
    id: RequestID,
    response_tx: tokio::sync::oneshot::Sender<ResponseToClientData>,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
struct RequestID {
    id: u64,
}

pub struct Client {
    request_tx: mpsc::UnboundedSender<ClientToRequestData>,
    index_done_tx: mpsc::UnboundedSender<()>,
    index_done_rx: Option<mpsc::UnboundedReceiver<()>>,
    semanctic_tokens_server: Option<lsp_types::SemanticTokensLegend>,
}
impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            request_tx: self.request_tx.clone(),
            index_done_tx: self.index_done_tx.clone(),
            index_done_rx: None,
            semanctic_tokens_server: self.semanctic_tokens_server.clone(),
        }
    }
}

pub struct ClangdClient {
    server_process: tokio::process::Child,
    request_id: RequestID,
}

struct Response {
    reader: BufReader<ChildStdout>,
    status: ResponseStatus,
}

struct ResponseRegister {
    id_map: HashMap<RequestID, ResponseRegisterData>,
}

enum ResponseRegisterData {
    Data(ResponseToClientData),
    Sender(tokio::sync::oneshot::Sender<ResponseToClientData>),
}

enum ResponseStatus {
    Init(Vec<u8>),
    ContentLength(usize, Vec<u8>),
    ContentLengthNext(Vec<u8>),
}

impl RequestID {
    fn id(&self) -> u64 {
        self.id
    }
}

impl ResponseRegister {
    pub fn register_data(&mut self, id: RequestID, data: ResponseToClientData) -> Result<()> {
        match self.id_map.entry(id.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let data_save = entry.remove();
                match data_save {
                    ResponseRegisterData::Sender(s) => {
                        s.send(data).expect("send responce to client fail");
                        Ok(())
                    }
                    ResponseRegisterData::Data(data_save) => {
                        error!("id {:?} have two data {data_save:?} {data:?}", &id);
                        Err(anyhow!("id {:?} have to data", &id))
                    }
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(ResponseRegisterData::Data(data));
                Ok(())
            }
        }
    }
    pub fn register_sender(
        &mut self,
        id: RequestID,
        sender: oneshot::Sender<ResponseToClientData>,
    ) -> Result<()> {
        match self.id_map.entry(id.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let data_save = entry.remove();
                match data_save {
                    ResponseRegisterData::Sender(s) => {
                        error!("id {:?} have two sander {s:?} {sender:?}", &id);
                        Err(anyhow!("id {:?} have to data", &id))
                    }
                    ResponseRegisterData::Data(data) => {
                        sender.send(data).expect("send responce to client fail");
                        Ok(())
                    }
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(ResponseRegisterData::Sender(sender));
                Ok(())
            }
        }
    }
}

impl Client {
    pub fn initialize(&self) -> Result<tokio::sync::oneshot::Receiver<ResponseToClientData>> {
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
        // Send initialize request
        self.request("initialize", params)
    }

    pub fn semantic_tokens_full(
        &mut self,
        file_path: &str,
    ) -> Result<oneshot::Receiver<ResponseToClientData>> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = lsp_types::SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.request("textDocument/semanticTokens/full", params)
    }

    pub fn hover(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<tokio::sync::oneshot::Receiver<ResponseToClientData>> {
        let uri = Uri::from_str(&format!("file://{}", file_path))?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        self.request("textDocument/hover", params)
    }

    pub fn initialized(&self) -> Result<()> {
        let method = "initialized";
        let params = lsp_types::InitializedParams {};
        self.notification(method, params)
    }

    pub fn did_open(&self, file_path: &str, content: &[String]) -> Result<()> {
        let method = "textDocument/didOpen";
        let uri = Uri::from_str(&format!("file://{}", file_path))?;
        let content = content.join("\n");
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "cpp".to_string(), // or "c" depending on file type
                version: 1,
                text: content,
            },
        };
        self.notification(method, params)
    }

    pub fn did_close(&self, file_path: &str) -> Result<()> {
        let method = "textDocument/didClose";
        let uri = Uri::from_str(&format!("file://{}", file_path))?;
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri },
        };
        self.notification(method, params)
    }

    pub async fn index_done(&mut self) -> Result<()> {
        match &mut self.index_done_rx {
            Some(rx) => rx
                .recv()
                .await
                .map_or(Err(anyhow!("not get index done")), |_| Ok(())),
            None => Err(anyhow!("this is cloned , cannot wait")),
        }
    }

    pub fn set_semantic_tokens_server(&mut self, legend: lsp_types::SemanticTokensLegend) {
        self.semanctic_tokens_server = Some(legend);
    }

    pub fn get_semantic_tokens_server(&self) -> Option<&lsp_types::SemanticTokensLegend> {
        self.semanctic_tokens_server.as_ref()
    }

    fn request<P: serde::Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<tokio::sync::oneshot::Receiver<ResponseToClientData>> {
        let (response_to_client_tx, response_to_client_rx) =
            oneshot::channel::<ResponseToClientData>();
        let valus = serde_json::json!(params);
        let data = ClientToRequestData::from((method, valus, response_to_client_tx));
        self.request_tx.send(data)?;
        Ok(response_to_client_rx)
    }

    fn notification<P: serde::Serialize>(&self, method: &str, params: P) -> Result<()> {
        let valus = serde_json::json!(params);
        let data = ClientToRequestData::from((method, valus));
        self.request_tx.send(data)?;
        Ok(())
    }

    fn response(&self, id: RequestID, method: &str, params: Value) -> Result<bool> {
        debug!(
            "client response method: {method} params :{:?}",
            params.clone()
        );
        match method {
            "window/workDoneProgress/create" => {
                debug!("in window/workDoneProgress/create");
                let params: lsp_types::WorkDoneProgressCreateParams =
                    serde_json::from_value(params)?;
                match params.token {
                    lsp_types::ProgressToken::String(token) => {
                        debug!("Progress created with token: {}", token);
                        let data = ClientToRequestData::from((method, Value::Null, id));
                        self.request_tx.send(data)?;
                    }
                    lsp_types::ProgressToken::Number(token) => {
                        debug!("Progress created with token: {}", token);
                    }
                }
            }
            "$/progress" => {
                let params: lsp_types::ProgressParams = serde_json::from_value(params)?;
                match params.value {
                    lsp_types::ProgressParamsValue::WorkDone(work_done) => match work_done {
                        lsp_types::WorkDoneProgress::Begin(begin) => {
                            info!(
                                "Progress begin: {} - {}",
                                begin.title,
                                begin.message.unwrap_or_default()
                            );
                        }
                        lsp_types::WorkDoneProgress::Report(report) => {
                            info!("Progress report: {}", report.message.unwrap_or_default());
                        }
                        lsp_types::WorkDoneProgress::End(end) => {
                            info!("Progress end: {}", end.message.unwrap_or_default());
                            self.index_done_tx.send(())?;
                            return Ok(true);
                        }
                    },
                }
            }
            _ => {
                // Unknown notification
                debug!("Unknown notification: {:?} {} {}", id, method, params);
            }
        }
        Ok(false)
    }
}

impl ClangdClient {
    pub async fn warpper_loop(mut self) -> Result<Client> {
        let stdin = self
            .server_process
            .stdin
            .take()
            .expect("Failed to open stdin");
        let stdout = self
            .server_process
            .stdout
            .take()
            .expect("Failed to open stdin");

        // stdin = self.initialize(stdin).await?;
        let (client_to_request_tx, client_to_request_rx) =
            tokio::sync::mpsc::unbounded_channel::<ClientToRequestData>();

        let (req_to_res_tx, req_to_res_rx) =
            tokio::sync::mpsc::unbounded_channel::<RequestToResponseData>();
        let client = Client::from((client_to_request_tx));
        tokio::spawn(Self::loop_response(stdout, client.clone(), req_to_res_rx));
        tokio::spawn(self.loop_request(stdin, client_to_request_rx, req_to_res_tx));
        Ok(client)
    }

    /// Start a new clangd server process and initialize the LSP connection
    pub fn new(log_path: &String, compile_commands_dir: String, debug: bool) -> Result<Self> {
        let log_file = std::fs::File::create(log_path)?;
        let log_level = match debug {
            true => "verbose",
            false => "info",
        };
        let jobs = std::thread::available_parallelism()
            .map(|n| n.get().saturating_mul(3))
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
            request_id: RequestID::from(()),
        };

        Ok(client)
    }
}

impl Response {
    async fn response(&mut self) -> Result<Value> {
        loop {
            match &mut self.status {
                ResponseStatus::Init(buf) => {
                    self.reader.read_until(b'\n', buf).await?;
                    let const_length = &buf[0.."Content-Length: ".len()];
                    let len = &buf["Content-Length: ".len()..buf.len() - 2];
                    let end = &buf[buf.len() - 2..];
                    assert_eq!(const_length, "Content-Length: ".as_bytes());
                    assert_eq!(end, "\r\n".as_bytes());
                    assert!(len.is_ascii());
                    let len = str::from_utf8(len)?.parse::<usize>()?;
                    debug!(
                        "response header: {}",
                        str::from_utf8(buf.as_slice()).unwrap()
                    );
                    self.status = ResponseStatus::ContentLength(len, vec![0_u8; 0]);
                }
                ResponseStatus::ContentLength(len, buf) => {
                    self.reader.read_until(b'\n', buf).await?;
                    assert_eq!(buf, "\r\n".as_bytes());
                    self.status = ResponseStatus::ContentLengthNext(vec![0_u8; len.clone()]);
                }
                ResponseStatus::ContentLengthNext(buf) => {
                    let point = buf.iter().position(|c| *c == 0).expect("must have 0");
                    self.reader.read_exact(&mut buf[point..]).await?;
                    debug!(
                        "response len: {} body: {}",
                        buf.len(),
                        str::from_utf8(buf.as_slice()).unwrap()
                    );
                    let response: serde_json::Value = serde_json::from_slice(&buf)?;
                    if let Some(error) = response.get("error") {
                        return Err(anyhow!("LSP error: {}", error));
                    }
                    self.status = ResponseStatus::Init(vec![]);
                    return Ok(response);
                }
            }
        }
    }
}

impl ClangdClient {
    async fn request_base<P: serde::Serialize>(
        &mut self,
        mut stdin: ChildStdin,
        id: Option<RequestID>,
        method: &str,
        params: P,
    ) -> Result<(ChildStdin, Option<RequestID>)> {
        let (request, id) = match (id, method) {
            (Some(id), "") => {
                // Create request JSON
                (
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id.id(),
                        "params": params
                    }),
                    Some(id),
                )
            }
            (Some(id), _) => {
                // Create request JSON
                (
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id.id(),
                    "method": method,

                        "params": params
                    }),
                    Some(id),
                )
            }
            (None, _) => (
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
        let id = self.request_id.clone();
        self.request_id.next();
        let (stdin, id) = self.request_base(stdin, Some(id), method, params).await?;
        id.map_or(Err(anyhow!("request must have id")), |id| Ok((stdin, id)))
    }

    async fn notification<P: serde::Serialize>(
        &mut self,
        stdin: ChildStdin,
        method: &str,
        params: P,
    ) -> Result<ChildStdin> {
        let (stdin, _) = self.request_base(stdin, None, method, params).await?;
        Ok(stdin)
    }

    async fn resopnse<P: serde::Serialize>(
        &mut self,
        stdin: ChildStdin,
        id: RequestID,
        params: P,
    ) -> Result<ChildStdin> {
        let (stdin, _) = self.request_base(stdin, Some(id), "", params).await?;
        Ok(stdin)
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
                Some(ClientToRequestData {
                    sender: ClientToRequestDataType::Request(sender),
                    method,
                    params,
                }) => match self.request(stdin, method.as_str(), params).await {
                    Ok((stdin_new, id)) => {
                        stdin = stdin_new;
                        match request_to_response_tx.send(RequestToResponseData::from((id, sender)))
                        {
                            Err(e) => {
                                error!("request fail {}", e);
                                break;
                            }
                            Ok(_) => {}
                        }
                    }
                    Err(e) => {
                        error!("request fail {} for {}", e, &method);
                        break;
                    }
                },
                Some(ClientToRequestData {
                    sender: ClientToRequestDataType::Notification,
                    method,
                    params,
                }) => match self.notification(stdin, method.as_str(), params).await {
                    Ok(stdin_new) => {
                        stdin = stdin_new;
                    }
                    Err(e) => {
                        error!("request fail {} for {}", e, method);
                        break;
                    }
                },
                Some(ClientToRequestData {
                    sender: ClientToRequestDataType::Response(id),
                    method,
                    params,
                }) => match self.resopnse(stdin, id, params).await {
                    Ok(stdin_new) => {
                        stdin = stdin_new;
                    }
                    Err(e) => {
                        error!("request fail {} for {}", e, method);
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
        client: Client,
        req_to_res_rx: tokio::sync::mpsc::UnboundedReceiver<RequestToResponseData>,
    ) -> () {
        let mut response = Response::from(BufReader::new(stdout));
        let mut response_register = ResponseRegister::from(());
        let mut req_to_res_rx = req_to_res_rx;
        loop {
            tokio::select! {
                value = response.response() => {
                    match value {
                        Ok(value)=>{
                            debug!("respose {:?}", &value);
                            let id_request = &value["id"];
                            let method = &value["method"];
                            let has_result_field = value.get("result").is_some();
                            let result = value.get("result").unwrap_or(&Value::Null);
                            if *id_request != serde_json::json!(null) && has_result_field {
                                let id_request = id_request.to_string();
                                let id_request = id_request
                                    .parse::<u64>()
                                    .expect("lsp server response not hav id");
                                let id = RequestID::from(id_request);
                                response_register
                                    .register_data(id, ResponseToClientData::from(result.clone()))
                                    .expect("register data fail");
                            } else if *method != serde_json::json!(null) {
                                let id_request = if *id_request != serde_json::json!(null) {
                                    let id_request = id_request.clone().to_string();
                                    id_request
                                        .parse::<u64>()
                                        .expect("lsp server response not hav id")
                                } else {
                                    0_u64
                                };
                                let params = value.get("params").expect("get params fail");
                                let method = value
                                    .get("method")
                                    .and_then(|m| m.as_str())
                                    .expect("get method fail");
                                client
                                    .response(RequestID::from(id_request), method, params.clone())
                                    .expect("handle server request fail");
                            }
                        },
                        Err(e)=>{
                            error!("response error {e}");
                            break;
                        }
                    }
                },
                value = req_to_res_rx.recv()=>{
                    // debug!("req to res {value:?}");
                    match value {
                        Some(data) => {
                            response_register
                                .register_sender(data.id, data.response_tx)
                                .expect("response_register fail");
                        }
                        None => {
                            error!("req_to_res close");
                            break;
                        }
                    }
                }
            }
        }
    }

    // fn handle_other_response()
}

impl RequestID {
    pub fn next(&mut self) {
        self.id += 1
    }
}

impl From<()> for RequestID {
    fn from(_: ()) -> Self {
        RequestID { id: 0 }
    }
}

impl From<u64> for RequestID {
    fn from(value: u64) -> Self {
        RequestID { id: value }
    }
}

impl From<(&str, Value, oneshot::Sender<ResponseToClientData>)> for ClientToRequestData {
    fn from(
        (method, params, sender): (&str, Value, oneshot::Sender<ResponseToClientData>),
    ) -> Self {
        ClientToRequestData {
            sender: ClientToRequestDataType::Request(sender),
            method: method.to_string(),
            params,
        }
    }
}

impl From<(&str, Value)> for ClientToRequestData {
    fn from((method, params): (&str, Value)) -> Self {
        ClientToRequestData {
            sender: ClientToRequestDataType::Notification,
            method: method.to_string(),
            params,
        }
    }
}

impl From<(&str, Value, RequestID)> for ClientToRequestData {
    fn from((method, params, id): (&str, Value, RequestID)) -> Self {
        ClientToRequestData {
            sender: ClientToRequestDataType::Response(id),
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

impl From<BufReader<ChildStdout>> for Response {
    fn from(reader: BufReader<ChildStdout>) -> Self {
        Response {
            reader,
            status: ResponseStatus::Init(vec![]),
        }
    }
}

impl From<()> for ResponseRegister {
    fn from(_: ()) -> Self {
        Self {
            id_map: HashMap::new(),
        }
    }
}

impl From<Value> for ResponseToClientData {
    fn from(value: Value) -> Self {
        Self { val: value }
    }
}

impl From<mpsc::UnboundedSender<ClientToRequestData>> for Client {
    fn from(sender: mpsc::UnboundedSender<ClientToRequestData>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<()>();
        Self {
            request_tx: sender,
            index_done_tx: tx,
            index_done_rx: Some(rx),
            semanctic_tokens_server: None,
        }
    }
}
