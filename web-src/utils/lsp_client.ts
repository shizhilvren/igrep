// import { Uri } from "monaco-editor";
import type { V86 } from "../../v86/v86";


export class ResponseLog {
    private status: 'header' | "split0" | 'body' = 'header';
    private len: number = 0;
    private client: LSPClient;
    private index_done: Promise<void>;
    private index_done_resolve: ((value: void | PromiseLike<void>) => void) | null = null;
    constructor(client: LSPClient) {
        this.client = client;
        this.index_done = new Promise((resolve) => {
            this.index_done_resolve = resolve;
        });
    }
    isBody() {
        return this.status === 'body'
    }
    public isIndexDone() {
        return this.index_done
    }
    public update(buf: string) {
        if (this.status === 'header') {
            if (buf.startsWith("Content-Length: ")) {
                const lengthStr = buf.substring("Content-Length: ".length);
                const length = parseInt(lengthStr, 10);
                if (!isNaN(length)) {
                    this.len = length;
                    this.status = 'split0';
                } else {
                    console.error('Failed to parse Content-Length:', lengthStr);
                }
            }
        } else if (this.status === 'split0') {
            if (buf === '') {
                this.status = 'body';
            } else {
                console.error('Expected newline while waiting for header split, got:', buf);
            }
        } else if (this.status === 'body') {
            if (buf.length >= this.len) {
                let new_buf = buf.substring(0, this.len);
                console.log('Received complete response:', new_buf);
                const json = JSON.parse(new_buf);
                const have_result = json.result !== undefined;
                const have_method = json.method !== undefined;
                const have_id = json.id !== undefined;
                const have_params = json.params != undefined;
                if (have_id && json.id === 0 && have_result && !have_method) {
                    this.client.initialized();
                    const name = "/";
                    this.client.didOpen(name, [""]);
                    this.client.didClose(name);
                } else if (have_id && !have_result && have_method) {
                    if (json.method === "window/workDoneProgress/create") {
                        this.client.resopnse(json.id, null);
                    }
                } else if (!have_id && !have_result && have_method && have_params) {
                    if (json.method == "$/progress") {
                        if (json.params.value.kind == "end") {
                            const fun = this.index_done_resolve!
                            fun();
                        }
                    }
                }
                const ret = buf.substring(this.len, buf.length)
                this.len = 0;
                this.status = 'header';
                return ret;
            } else {
                return buf;
            }
        }
        return '';
    }
}

export class LSPClient {
    private static readonly HEADER_SEPARATOR_BYTES = new Uint8Array([10, 10]);

    constructor(v86: V86) {
        this.v86 = v86;
    }
    private buf: string = '';
    private v86: V86;
    private id: number = 0;
    private response_log: ResponseLog = new ResponseLog(this);
    public update(byte: number) {
        const char = String.fromCharCode(byte);
        if (char === '\n') {
            while (true) {
                if (this.isLog()) {
                    console.debug('log from LSP server:', this.buf);
                    break;
                } else {
                    this.buf = this.response_log.update(this.buf);
                    if (this.buf === "") {
                        break;
                    }
                }
            }
            this.next();
            this.buf = '';
        } else if (char != '\r') {
            this.buf += char;
            if (this.response_log.isBody()) {
                this.buf = this.response_log.update(this.buf);
            }
        }
    }



    public waitForEmulatorReady(): Promise<boolean> {
        return new Promise((resolve) => {
            let settled = false;

            const cleanup = (): void => {
                this.v86.remove_listener("emulator-ready", onReady);
            };

            const finish = (ready: boolean): void => {
                if (settled) {
                    return;
                }
                settled = true;
                cleanup();
                resolve(ready);
            };

            const onReady = (): void => finish(true);

            this.v86.add_listener("emulator-ready", onReady);
        });
    }
    public start() {
        this.v86.serial0_send("stty raw -echo\n");
        const cmd = [
            "clangd",
            "--compile-commands-dir=/lsp/",
            "--log=info",
            "\n"
        ].join(" ");
        this.v86.serial0_send(cmd);
    }

    public isIndexDone() {
        return this.response_log.isIndexDone()
    }
    next() {
        if (this.buf.endsWith("Starting LSP over stdin/stdout")) {
            this.initialize()
        }
    }

    initialize(): void {
        this.request(
            "initialize",
            { "capabilities": { "textDocument": { "codeLens": { "dynamicRegistration": true }, "definition": { "dynamicRegistration": true, "linkSupport": true }, "documentSymbol": { "dynamicRegistration": true, "hierarchicalDocumentSymbolSupport": true, "symbolKind": { "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26] }, "tagSupport": { "valueSet": [1] } }, "hover": { "contentFormat": ["markdown", "plaintext"], "dynamicRegistration": true }, "semanticTokens": { "augmentsSyntaxTokens": true, "dynamicRegistration": true, "formats": ["relative"], "multilineTokenSupport": false, "overlappingTokenSupport": false, "requests": { "full": true }, "tokenModifiers": ["declaration", "definition", "readonly", "static", "deprecated", "abstract", "async", "modification", "documentation", "defaultLibrary"], "tokenTypes": ["namespace", "type", "class", "enum", "interface", "struct", "typeParameter", "parameter", "variable", "property", "enumMember", "event", "function", "method", "macro", "keyword", "modifier", "comment", "string", "number", "regexp", "operator", "decorator"] } }, "window": { "workDoneProgress": true } }, "clientInfo": { "name": "igrep-clangd-client", "version": "0.1.0" }, "processId": 2482212, "rootUri": null, "trace": "verbose" }
        )
    }

    initialized() {
        this.notify("initialized", {})
    }

    didOpen(file_path: string, content: string[]) {
        const method = "textDocument/didOpen";
        let uri = "file://" + file_path;
        const content_str = content.join("\n");
        const params = {
            textDocument: {
                uri: uri.toString(),
                languageId: "cpp",
                version: 1,
                text: content_str
            }
        };
        this.notify(method, params);
    }

    didClose(file_path: string) {
        const method = "textDocument/didClose";
        let uri = "file://" + file_path;
        const params = {
            textDocument: {
                uri: uri.toString(),
            }
        };
        this.notify(method, params);
    }
    request(method: string, params: Record<string, unknown>): void {
        const id = this.id++;
        const json = { id, jsonrpc: "2.0", method, params };
        const str = JSON.stringify(json);
        const len = new TextEncoder().encode(str).length;
        const header = `Content-Length: ${len}`;

        // 使用 serial_send_bytes 发送 header + \r\n
        const headerBytes = new TextEncoder().encode(header);
        this.v86.serial_send_bytes(0, headerBytes);

        // 发送 \r\n\r\n 分隔符
        this.v86.serial_send_bytes(0, LSPClient.HEADER_SEPARATOR_BYTES);

        // 发送 JSON body
        const bodyBytes = new TextEncoder().encode(str);
        this.v86.serial_send_bytes(0, bodyBytes);

        // this.v86.serial_send_bytes(0, headerBytes);
        this.v86.serial_send_bytes(0, new Uint8Array([10]));
        // this.v86.serial_send_bytes(0, bodyBytes);
    }

    resopnse(id: number, params: JSON | null) {
        const json = { id, jsonrpc: "2.0", params };
        const str = JSON.stringify(json);
        const len = new TextEncoder().encode(str).length;
        const header = `Content-Length: ${len}`;

        // 使用 serial_send_bytes 发送 header + \r\n
        const headerBytes = new TextEncoder().encode(header);
        this.v86.serial_send_bytes(0, headerBytes);

        // 发送 \r\n\r\n 分隔符
        this.v86.serial_send_bytes(0, LSPClient.HEADER_SEPARATOR_BYTES);

        // 发送 JSON body
        const bodyBytes = new TextEncoder().encode(str);
        this.v86.serial_send_bytes(0, bodyBytes);

        this.v86.serial_send_bytes(0, new Uint8Array([10]));
    }

    notify(method: string, params: Record<string, unknown>): void {
        const json = { jsonrpc: "2.0", method, params };
        const str = JSON.stringify(json);
        const len = new TextEncoder().encode(str).length;
        const header = `Content-Length: ${len}`;

        // 使用 serial_send_bytes 发送 header + \r\n
        const headerBytes = new TextEncoder().encode(header);
        this.v86.serial_send_bytes(0, headerBytes);

        // 发送 \r\n\r\n 分隔符
        this.v86.serial_send_bytes(0, LSPClient.HEADER_SEPARATOR_BYTES);

        // 发送 JSON body
        const bodyBytes = new TextEncoder().encode(str);
        this.v86.serial_send_bytes(0, bodyBytes);

        this.v86.serial_send_bytes(0, new Uint8Array([10]));
    }


    isLog(): boolean {
        return this.isInfo() || this.isWarning() || this.isError() || this.isVerbose();
    }
    isInfo(): boolean {
        return this.buf.startsWith("I[");
    }
    isVerbose(): boolean {
        return this.buf.startsWith("V[");
    }
    isWarning(): boolean {
        return this.buf.startsWith("W[");
    }
    isError(): boolean {
        return this.buf.startsWith("E[");
    }
}
