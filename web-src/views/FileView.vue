<script setup lang="ts">
import FilePathBar from '@/components/lsp/FilePathBar.vue';
import OneFile from '@/components/lsp/OneFile.vue';
import { languages } from 'monaco-editor';

const props = defineProps<{
    filePath: string | string[]
}>()

let code = [
    "fn main() {",
    "    let name = \"igrep\";",
    "    println!(\"hello {}\", name);",
    "}"
]

</script>

<template>
    <main class="file-view">
        <div style="padding-left: 10px;">
            <FilePathBar v-bind="{ filePath: Array.isArray(filePath) ? filePath : [filePath] }" />
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
