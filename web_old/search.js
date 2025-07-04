import { ref, defineComponent } from 'vue'
import igrep_init, { Engine } from "./pkg/igrep.js";

// 定义文件路径常量
const data_file_path = "index/igrep.dat";
const index_file_path = "index/igrep.idx";
// 获取索引文件数据的Promise
let p1 = fetchFileToUint8Array(index_file_path);
// 响应式状态变量
const count = ref(0)
const msg = ref("")
const init_finish = ref(false)
const engine = ref(null)



export default defineComponent({
    name: 'SearchComponent', // 添加名称便于调试
    setup() {
        console.log("rust load finish")
        return { msg, count, init_finish, search }
    },
    async mounted() {
        await rust_api_init();
    },
    template: `
    load index finsihs {{ init_finish }}
<p>Message is: {{ msg }}</p>
<input v-model="msg" placeholder="edit me" class="form-control" type="email"/>
<button @click="search(msg)" type="submit" class="btn btn-primary">Search</button>  
`
})



async function search(msg) {
    console.log(`searching "${msg}!"`)
    try {
        // 检查 engine 是否已初始化
        if (!engine.value) {
            alert('Search engine is not initialized');
            return;
        }

        // 安全地调用 engine 的方法
        const result = await engine.value.regex(msg);

    } catch (error) {
        console.error('Search error:', error);
        alert(`Search error: ${error.message}`);
    }
}

async function rust_api_init() {
    try {
        let p2 = igrep_init();
        let [index_file_data, _] = await Promise.allSettled([p1, p2]);

        // 检查 index_file_data 是否成功获取
        if (index_file_data.status === 'fulfilled' && index_file_data.value) {
            // 确保使用正确的类型创建 Engine 实例
            engine.value = await new Engine(index_file_data.value);
            init_finish.value = true;
            console.log("engine init finish");
        } else {
            // 类型断言，因为我们知道如果status不是fulfilled，就是rejected
            const rejected = index_file_data;
            throw new Error('Failed to load index data: ' +
                (rejected.reason instanceof Error ? rejected.reason.message : 'Unknown error'));
        }
    } catch (error) {
        console.error('Engine initialization error:', error);
        alert(`Failed to initialize search engine: ${error.message}`);
    }
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
        // 确保重新抛出的是Error对象
        if (error instanceof Error) {
            throw error;
        }
        throw new Error('Unknown error during file range fetch');
    }
}

