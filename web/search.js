import { ref } from 'vue'
import igrep_init, { build_index_data, index_regex_engine, engine_build_tree } from "./pkg/igrep.js";

const index_file_path = "index/igrep.idx";
const data_file_path = "index/igrep.dat";
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
        let range = [0..len];
        for (let i = 0; i < len; i++) {
            let r = tree.range_at(i);
            console.log("range " + i + range)
        }
        console.log("need get " + len + " ngram range")
    } else {
        alert('"${msg}" is not a regex')
    }
}

async function rust_api_init() {
    await igrep_init();
    igrep_index_data.value = await load_index_data();
    init_finish.value = true;
    return
}


async function load_index_data() {
    let index_file_data = await fetchFileToUint8Array(index_file_path);
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