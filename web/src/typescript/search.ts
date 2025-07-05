import _, { FileDataMatchRange } from "../../pkg/igrep"

export class LineResult {
    line: number;
    string: string;
    match: Array<FileDataMatchRange>;
    constructor(line: number, string: string, match: Array<FileDataMatchRange>) {
        this.line = line;
        this.string = string
        this.match = match
    }
}

export class FileLinesResult {
    name: string;
    lines: Array<LineResult>;
    constructor(name: string, lines: Array<LineResult>) {
        this.name = name
        this.lines = lines
    }
}


export async function fetchFileToUint8Array(url: URL, signal?: AbortSignal): Promise<Uint8Array> {
    try {
        // 发起网络请求
        const response = await fetch(url, signal ? { signal } : {});

        // 检查响应状态
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        // 获取 ArrayBuffer
        const arrayBuffer = await response.arrayBuffer();

        // 转换为 Uint8Array (相当于 Rust 的 Vec<u8>)
        return new Uint8Array(arrayBuffer);
    } catch (error) {
        console.error('Error fetching file:', error);
        // 确保重新抛出的是Error对象
        if (error instanceof Error) {
            throw error;
        }
        throw new Error('Unknown error during file fetch');
    }
}


/**
 * 从网络上读取文件的特定范围，支持u64大小的文件
 * @param url - 文件的URL
 * @param start - 起始字节位置（包含），支持u64大小的值
 * @param len - 要读取的字节长度，支持u64大小的值
 * @returns 包含请求范围内数据的Uint8Array
 */
export async function fetchFileRange(url: URL, start: bigint, len: bigint, signal?: AbortSignal): Promise<Uint8Array> {
    try {
        // 确保使用BigInt处理，以支持超过2^53-1的值
        const startBig = BigInt(start);
        const lenBig = BigInt(len);
        const endBig = startBig + lenBig - 1n; // 计算结束位置

        // 创建带有Range头的请求
        const headers = new Headers();
        headers.append('Range', `bytes=${startBig.toString()}-${endBig.toString()}`);

        // 发起网络请求
        const response = await fetch(url, { headers, signal });

        // 检查响应状态
        if (!response.ok && response.status !== 206) {
            // 206是部分内容的HTTP状态码
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        // 获取ArrayBuffer
        const arrayBuffer = await response.arrayBuffer();

        // 转换为Uint8Array
        return new Uint8Array(arrayBuffer);
    } catch (error) {
        // console.error('Error fetching file range:', error);
        // console.error(`Failed range request: ${url} from ${start} length ${len}`);
        // 确保重新抛出的是Error对象
        if (error instanceof Error) {
            throw error;
        }
        throw new Error('Unknown error during file range fetch');
    }
}