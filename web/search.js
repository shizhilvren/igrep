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


async function search(msg) {
    console.log(`searching "${msg}!"`)
    let engine = index_regex_engine(msg)
    if (engine) {
        let tree = engine_build_tree(engine, igrep_index_data.value, 3)
        let len = tree.get_len();
        console.log("need get " + len + " ngram range")
        let p_array = Array.from({ length: len }, (_, i) => {
            let r = tree.range_at(i);
            let len = r[0].len; // len is inclusive
            let start = r[0].start; // start is inclusive
            console.log("range " + i + " " + start + " " + r[0].len)
            return fetchFileRange(data_file_path, start, len)
        });
        let data = await Promise.all(p_array)
        console.log("get ngram data finish")
        console.log(data)
        for (let i = 0; i < data.length; i++) {
            console.debug("set data at " + i + " start")
            let v = data.at(i);
            tree.set_data_at(i, v);
            console.debug("set data at " + i + " finish ")

        }
        console.log("get ngram data finish")
        let ngram_tree_result = tree.search()
        if (ngram_tree_result.all()) {
            alert("this regex samil then 3")
        } else {
            console.log("have " + ngram_tree_result.len() + " may result")
            for (let i = 0; i < ngram_tree_result.len(); i++) {
                let file_line_index = ngram_tree_result.get(i)
                console.log("file index " + file_line_index.file_id + " line " + file_line_index.line_id.line_number())
            }
        }
        console.log("file line search finish")
    } else {
        alert('"${msg}" is not a regex')
    }
}

async function rust_api_init() {
    let p2 = igrep_init();
    let [index_file_data, _] = await Promise.allSettled([p1, p2])
    igrep_index_data.value = await load_index_data(index_file_data.value);
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
 * 从网络上读取文件的特定范围，支持u64大小的文件
 * @param {string} url - 文件的URL
 * @param {number|BigInt} start - 起始字节位置（包含），支持u64大小的值
 * @param {number|BigInt} len - 要读取的字节长度，支持u64大小的值
 * @returns {Promise<Uint8Array>} - 包含请求范围内数据的Uint8Array
 */
async function fetchFileRange(url, start, len) {
    try {
        // 确保使用BigInt处理，以支持超过2^53-1的值
        const startBig = BigInt(start);
        const lenBig = BigInt(len);
        const endBig = startBig + lenBig - 1n; // 计算结束位置
        
        // 创建带有Range头的请求
        const headers = new Headers();
        headers.append('Range', `bytes=${startBig.toString()}-${endBig.toString()}`);
        
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
        console.error(`Failed range request: ${url} from ${start} length ${len}`);
        throw error;
    }
}

