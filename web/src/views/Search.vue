<script setup lang="ts">
import { ref, defineComponent, onMounted } from 'vue'
import igrep_init, { Engine, NgreamIndexData, FileDataMatchRange } from "../../pkg/igrep"

// 定义文件路径常量
const data_file_path: URL = new URL("../../igrep/igrep.dat", import.meta.url);
const index_file_path: URL = new URL("../../igrep/igrep.idx", import.meta.url);
let p1 = fetchFileToUint8Array(index_file_path);

onMounted(async () => {
    await load_igrep()

})

defineProps<{
}>()

const init_finish = ref(false)
const msg = ref("")
const engine = ref<Engine | null>(null)
const search_result = ref<Array<LineResult>>([])

class LineResult {
    line: number;
    string: string;
    match: Array<FileDataMatchRange>;
    constructor(line: number, string: string, match: Array<FileDataMatchRange>) {
        this.line = line;
        this.string = string
        this.match = match
    }
}

async function search(msg: string) {
    search_result.value = []
    console.debug("start search {}", msg)
    if (engine.value !== null) {
        let engine_local = engine.value
        let ngram_tree = engine_local.regex(msg)
        let ngrams = engine_local.ngram_ranges(ngram_tree);
        let ngram_datas_buf = await Promise.all(ngrams.map(ngram => {
            let len = BigInt(ngram.range[0].len);
            return fetchFileRange(data_file_path, ngram.range[0].start, len)
        }))
        console.debug("all ngrams data get finsih number {}", ngram_datas_buf.length)
        let ngram_datas = ngram_datas_buf.map(function (e, i) {
            return new NgreamIndexData(ngrams[i].index(), e)
        })
        console.debug("all ngrams data build finsih number {}", ngram_datas.length)
        let ngram_tree_result_struct = engine_local.get_search_result(ngram_tree, ngram_datas)
        console.debug("result calulate finish")
        if (ngram_tree_result_struct.is_all()) {
            alert("search len smail them 3")
        } else {
            let data = ngram_tree_result_struct.file_lines();
            for (let id of data.keys()) {
                let file_lines = data[id]
                let r = engine_local.file_range(file_lines.file)
                if (r !== undefined) {
                    let file_data_buf = await fetchFileRange(data_file_path, r[0].start, BigInt(r[0].len))
                    console.debug("start get file result {}", id)
                    let file_data = engine_local.build_file_data(file_data_buf)
                    let file_name = file_data.name()
                    let line_reault = await Promise.all(file_lines.lines().map(async file_line => {
                        let r = file_data.lines_range(file_line)
                        if (r !== undefined) {
                            let line_data_buf = await fetchFileRange(data_file_path, r[0].start, BigInt(r[0].len))
                            let line_data = engine_local.build_file_line_data(line_data_buf)
                            let match_ranges =
                                engine_local.file_data_match(line_data, ngram_tree_result_struct);
                            return new LineResult(
                                file_line.line_number(), line_data.get(), match_ranges
                            )
                        } else {
                            throw new Error('Unknown error during file_line fetch');
                        }

                    }));
                    line_reault.forEach(a => {
                        if (a.match.length != 0) {
                            search_result.value.push(a)
                            console.log(a)
                        }
                    })
                    console.debug("finish get file result {}", id)

                } else {
                    throw new Error('Unknown error during file fetch');
                }

            }


        }
    }
}

async function load_igrep() {
    try {
        let p2 = igrep_init();
        let [index_file_data, _] = await Promise.allSettled([p1, p2]);

        // 检查 index_file_data 是否成功获取
        if (index_file_data.status === 'fulfilled') {
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
    } catch (error: any) {
        console.error('Engine initialization error:', error);
        alert(`Failed to initialize search engine: ${error.message}`);
    }
}


async function fetchFileToUint8Array(url: URL) {
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
async function fetchFileRange(url: URL, start: bigint, len: bigint) {
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

</script>

<template>
    <div>
        load index finsihs {{ init_finish }}
        <p>Message is: {{ msg }}</p>
        <el-input v-model="msg" :disabled="!init_finish" style="width: 240px" placeholder="Please input" />
        <el-button @click="search(msg)" type="primary" class="btn btn-primary">Search</el-button>
        <p>{{ search_result }}</p>
    </div>
</template>


<style scoped>
h1 {
    font-weight: 500;
    font-size: 2.6rem;
    position: relative;
    top: -10px;
}

h3 {
    font-size: 1.2rem;
}

.greetings h1,
.greetings h3 {
    text-align: center;
}

@media (min-width: 1024px) {

    .greetings h1,
    .greetings h3 {
        text-align: left;
    }
}
</style>
