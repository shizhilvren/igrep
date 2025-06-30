import { ref } from 'vue'
import igrep_init, { build_index_data, index_regex_engine, engine_build_tree } from "./pkg/igrep.js";

const data_file_path = "index/igrep.dat";
const index_file_path = "index/igrep.idx";
let p1 = fetchFileToUint8Array(index_file_path);
const count = ref(0)
const msg = ref("")
const init_finish = ref(false)
const igrep_index_data = ref("")
const ngram_len = ref(4)



export default {
    setup() {
        console.log("rust load finish")
        return { msg, count, init_finish, search }
    },
    async mounted() {
        await rust_api_init();
        console.log(`the component is now mounted.`)
    },
    template: `
    load index finsihs {{ init_finish }}
<p>Message is: {{ msg }}</p>
<input v-model="msg" placeholder="edit me" class="form-control" type="email"/>
<button @click="search(msg)" type="submit" class="btn btn-primary">Search</button>  
`
}


function search(msg) {
    console.log(`searching "${msg}!"`)
    let engine = index_regex_engine(msg)
    if (engine) {
        let tree = engine_build_tree(engine, igrep_index_data.value, 3)
        let len = tree.get_len();
        let p_array = Array.from({ length: len }, (_, i) => {
            let r = tree.range_at(i);
            let end = r[0].start + r[0].len - 1; // end is inclusive
            let start = r[0].start; // start is inclusive
            console.log("range " + i + start + " " + r[0].len)
            return fetchFileRange(data_file_path, start, end)
        });
        Promise.all(p_array).then((data) => {
            console.log("get ngram data finish")
            console.log(data)
        });
        console.log("need get " + len + " ngram range")
    } else {
        alert('"${msg}" is not a regex')
    }
}

async function rust_api_init() {
    let p2 = igrep_init();
    let [index_file_data, _] = await Promise.all([p1, p2])
    igrep_index_data.value = await load_index_data(index_file_data);
    init_finish.value = true;
    return
}


async function load_index_data(index_file_data) {
    let index_data = build_index_data(index_file_data);
    console.log("get index data finish");
    return index_data;
}

async function fetchFileToUint8Array(url) {
    try {
        // 发起网络请求
        const response = await fetch(url);

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
        throw error; // 可以选择重新抛出或返回空数组
    }
}

/**
 * 从网络上读取文件的特定范围
 * @param {string} url - 文件的URL
 * @param {number} start - 起始字节位置（包含）
 * @param {number} end - 结束字节位置（包含）
 * @returns {Promise<Uint8Array>} - 包含请求范围内数据的Uint8Array
 */
async function fetchFileRange(url, start, end) {
    try {
        // 创建带有Range头的请求
        const headers = new Headers();
        headers.append('Range', `bytes=${start}-${end}`);

        // 发起网络请求
        const response = await fetch(url, { headers });

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
        console.error('Error fetching file range:', error);
        throw error;
    }
}

