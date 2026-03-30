import { CancellablePromise } from 'real-cancellable-promise';

export async function fetchFileData(path_base_url: String, controller?: AbortController): Promise<Uint8Array | undefined> {
    let header = new Headers();
    // header.append("Content-Type", "application/octet-stream");
    let name = import.meta.env.BASE_URL + path_base_url;
    let url = new URL(name, import.meta.url);
    // console.log('Fetching file data from URL:', url);
    // console.log('Fetching file data from URL:', url.toString());
    let nurmal_fetch = fetch(url.toString(), { headers: header, signal: controller?.signal });
    try {

        let res = await nurmal_fetch;
        let buffer = await res.arrayBuffer();
        return new Uint8Array(buffer);
    } catch (err) {
        if (err instanceof DOMException && err.name === 'AbortError') {
            console.log('Fetch aborted for URL:', url);
            return undefined;
        } else {
            throw err;
        }
    }


}

export class OneLineRange {
    constructor(startCollNumber: number, endCollNumber: number) {
        this.startCollNumber = startCollNumber;
        this.endCollNumber = endCollNumber;
    }
    readonly startCollNumber: number;
    readonly endCollNumber: number;
}