export async function fetchFileData(path_base_url: String): Promise<Uint8Array> {
    let name = import.meta.env.BASE_URL + path_base_url;
    let url = new URL(name, import.meta.url);
    const response = await fetch(url.toString());
    const arrayBuffer = await response.arrayBuffer();
    return new Uint8Array(arrayBuffer);
}