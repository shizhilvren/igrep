<script setup lang="ts">
import FilePathBar from '@/components/lsp/FilePathBar.vue';
import OneFile from '@/components/lsp/OneFile.vue';
import { languages } from 'monaco-editor';
import { onMounted, ref, watch } from 'vue';
import { PathIndex } from 'igrep';
import { fetchFileData } from "@/utils/utils"

const props = defineProps<{
    filePath: string | string[]
}>()
const path = ref<string[]>([]);

onMounted(() => {
    path.value = Array.isArray(props.filePath) ? props.filePath : [props.filePath];
    let path_str = path.value.join("/");
    let path_index = new PathIndex(path_str);
    let data = fetchFileData(path_index.path_str("lsp-index") + "/tree.data");
    console.log("file data", data);
})

let code = [
    "fn main() {",
    "    let name = \"igrep\";",
    "    println!(\"hello {}\", name);",
    "}"
]
watch(path, (new_path) => {
    console.log(`x is ${new_path}`)
})
</script>

<template>
    <main class="file-view">
        <div style="padding-left: 10px;">
            <FilePathBar v-bind="{ filePath: path }" />
        </div>
        <div class="file-editor">
            <OneFile v-bind="{ language: 'rust', code: code }" />
        </div>
    </main>
</template>

<style scoped>
.file-view {
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.file-editor {
    flex: 1;
    min-height: 0;
}
</style>
