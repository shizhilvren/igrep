<script setup lang="ts">
import { ref, defineComponent, onMounted } from 'vue'
import igrep_init, { Engine, NgreamIndexData, FileDataMatchRange } from "../../pkg/igrep"
import ResultFile from './ResultFile.vue';
import { FileLinesResult, LineResult, fetchFileToUint8Array, fetchFileRange } from '../typescript/search.ts';
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
const msg = ref("gdb_init \\(")
const engine = ref<Engine | null>(null)
const search_result = ref<Array<FileLinesResult>>([])
const searching = ref<{ fun: Promise<void>, ctrl: AbortController }>()

async function search(msg: string) {
    let controller = new AbortController();
    const signal = controller.signal;
    if (searching.value !== undefined) {
        searching.value.ctrl.abort("user search next regex");
    }
    searching.value = { fun: search_one(msg, signal), ctrl: controller }
    try {
        await searching.value.fun;
    } catch (e) {
        console.debug("searching abort by user {}", e)
    }
}

async function search_one(msg: string, signal: AbortSignal) {
    search_result.value = []
    console.debug("start search {}", msg)
    if (engine.value !== null) {
        let engine_local = engine.value
        let ngram_tree = engine_local.regex(msg)
        let ngrams = engine_local.ngram_ranges(ngram_tree);
        let ngram_datas_buf = await Promise.all(ngrams.map(ngram => {
            let len = BigInt(ngram.range[0].len);
            return fetchFileRange(data_file_path, ngram.range[0].start, len, signal).catch(e => {
                throw new Error('Failed to fetch ngram data: ' + e);
            });

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

                    let file_data_buf = await fetchFileRange(data_file_path, r[0].start, BigInt(r[0].len), signal).catch(e => {
                        throw new Error('Failed to fetch file data: ' + e);
                    });
                    console.debug("start get file result {}", id)
                    let file_data = engine_local.build_file_data(file_data_buf)
                    let file_name = file_data.name()
                    let lines_reault = await Promise.all(file_lines.lines().map(async file_line => {
                        let r = file_data.lines_range(file_line)
                        if (r !== undefined) {
                            let line_data_buf = await fetchFileRange(data_file_path, r[0].start, BigInt(r[0].len), signal).catch(e => {
                                throw new Error('Failed to fetch file line data: ' + e);
                            });
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
                    let line_reault_filter = lines_reault.filter(line => {
                        // console.log(line,) 
                        return line.match.length != 0
                    })
                    if (line_reault_filter.length != 0) {
                        let a = new FileLinesResult(
                            file_name, line_reault_filter
                        )

                        search_result.value.push(a)
                        console.debug("have result {}", line_reault_filter.length)
                    }
                    console.debug("finish get file result {}", id)

                } else {
                    throw new Error('Unknown error during file fetch');
                }
            }
        }
        console.log("search finish")
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

</script>

<template>
    <div>
        <div>
            <!-- load index finsihs {{ init_finish }} -->
            <!-- <p>Message is: {{ msg }}</p> -->

            <el-row>
                <el-col :span="22" class="alignment-container">
                    <el-input v-model="msg" :disabled="!init_finish" style="width: 100%" placeholder="Please input" />
                </el-col>
                <el-col :span="2">
                    <el-button @click="search(msg)" type="primary" class="btn btn-primary"
                        :disabled="!init_finish" style="width: 100%;">Search</el-button>
                </el-col>
            </el-row>
        </div>
        <div>
            <p>list of result </p>
            <el-scrollbar height="100%">
                <p v-for="item in search_result" class="scrollbar-demo-item">
                    <ResultFile :file_line_result=item />
                </p>
            </el-scrollbar>
        </div>
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
