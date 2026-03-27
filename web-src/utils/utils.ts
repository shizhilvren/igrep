export async function fetchFileData(path_base_url: String): Promise<Uint8Array> {
    let header = new Headers();
    // header.append("Content-Type", "application/octet-stream");
    let name = import.meta.env.BASE_URL + path_base_url;
    let url = new URL(name, import.meta.url);
    console.log('Fetching file data from URL:', url);
    console.log('Fetching file data from URL:', url.toString());
    const response = await fetch(url.toString(), { headers: header });
    const arrayBuffer = await response.arrayBuffer();
    return new Uint8Array(arrayBuffer);
}

export class OneLineRange {
    constructor(startCollNumber: number, endCollNumber: number) {
        this.startCollNumber = startCollNumber;
        this.endCollNumber = endCollNumber;
    }
    readonly startCollNumber: number;
    readonly endCollNumber: number;
}