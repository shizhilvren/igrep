import type { V86 } from "../../v86/v86";

export class LSPClient {
    private static readonly HEADER_SEPARATOR_BYTES = new Uint8Array([10, 10]);

    constructor(v86: V86) {
        this.v86 = v86;
    }
    private buf: string = '';
    private v86: V86;
    private id: number = 0;
    public update(byte: number) {
        const char = String.fromCharCode(byte);
        if (char === '\n') {
            console.log('Received line from LSP server:', this.buf, byte);
            this.next();
            this.buf = '';
        } else if (char != '\r') {
            this.buf += char;
        }
    }

    public waitForEmulatorReady(timeoutMs = 20000): Promise<boolean> {
        return new Promise((resolve) => {
            let settled = false;

            const cleanup = (): void => {
                this.v86.remove_listener("emulator-ready", onReady);
                clearTimeout(timer);
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
            const timer = setTimeout(() => finish(false), timeoutMs);

            this.v86.add_listener("emulator-ready", onReady);
        });
    }
    public start() {
        this.v86.serial0_send("stty -echo\n");
        this.v86.serial0_send("clangd --log=verbose\n");
    }
    next() {
        if (this.buf.endsWith("Starting LSP over stdin/stdout")) {
            this.init()
        }
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
    init(): void {
        this.request(
            "initialize",
            { "capabilities": { "textDocument": { "codeLens": { "dynamicRegistration": true }, "definition": { "dynamicRegistration": true, "linkSupport": true }, "documentSymbol": { "dynamicRegistration": true, "hierarchicalDocumentSymbolSupport": true, "symbolKind": { "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26] }, "tagSupport": { "valueSet": [1] } }, "hover": { "contentFormat": ["markdown", "plaintext"], "dynamicRegistration": true }, "semanticTokens": { "augmentsSyntaxTokens": true, "dynamicRegistration": true, "formats": ["relative"], "multilineTokenSupport": false, "overlappingTokenSupport": false, "requests": { "full": true }, "tokenModifiers": ["declaration", "definition", "readonly", "static", "deprecated", "abstract", "async", "modification", "documentation", "defaultLibrary"], "tokenTypes": ["namespace", "type", "class", "enum", "interface", "struct", "typeParameter", "parameter", "variable", "property", "enumMember", "event", "function", "method", "macro", "keyword", "modifier", "comment", "string", "number", "regexp", "operator", "decorator"] } }, "window": { "workDoneProgress": true } }, "clientInfo": { "name": "igrep-clangd-client", "version": "0.1.0" }, "processId": 2482212, "rootUri": null, "trace": "verbose" }
        )
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
}