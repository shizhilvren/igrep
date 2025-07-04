<script setup lang="ts">
import { ref, defineComponent, onMounted } from 'vue'
import igrep_init, { Engine } from "../../pkg/igrep"

// 定义文件路径常量
const data_file_path = "../../igrep/igrep.dat";
const index_file_path = "../../igrep/igrep.idx";
let p1 = fetchFileToUint8Array(index_file_path);

onMounted(async () => {
    let p1 = load_igrep()
    await fetchFileToUint8Array(index_file_path)

})

defineProps<{
}>()

const init_finish = ref(false)
const msg = ref("")
const engine = ref<Engine | null>(null)


async function search(msg: String) {

}

async function load_igrep() {
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
</script>

<template>
    <div>
        load index finsihs {{ init_finish }}
        <p>Message is: {{ msg }}</p>
        <el-input v-model="msg" :disabled="!init_finish" style="width: 240px" placeholder="Please input" />
        <el-button @click="search(msg)" type="primary" class="btn btn-primary">Search</el-button>
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
