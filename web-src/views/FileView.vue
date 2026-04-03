<template>
    <main class="file-view">
        <div style="padding-left: 10px;">
            <FilePathBar v-bind="{ filePath: normalizedPath }" />
        </div>
        <DirTree v-if="is_dir" v-bind="{
            dirs: dir_data.dirs,
            files: dir_data.files,
            base_path: normalizedPath
        }" />
        <div v-if="is_file" class="file-editor">
            <OneFile v-bind="{ language: 'rust', code: code }" />
        </div>
    </main>
</template>

<script setup lang="ts">
import FilePathBar from '@/components/lsp/FilePathBar.vue';
import OneFile from '@/components/lsp/OneFile.vue';
import { computed, ref, watch } from 'vue';
import { PathIndex, TreeData } from 'igrep';
import { fetchFileData } from "@/utils/utils"
import DirTree from '@/components/lsp/DirTree.vue';

const props = defineProps<{
    filePath: string | string[]
}>()

const dir_data = ref<DirData>(new DirData([], []))
const code = ref<string[]>([])
const is_dir = ref(false)
const is_file = ref(false)

function normalizeFilePath(filePath: string | string[]): string[] {
    const segments = Array.isArray(filePath) ? filePath : [filePath]
    return segments
        .flatMap((segment) => segment.split('/'))
        .map((segment) => segment.trim())
        .filter((segment) => segment.length > 0)
}

function isSamePath(a: string[], b: string[]): boolean {
    return a.length === b.length && a.every((value, index) => value === b[index])
}

const normalizedPath = computed<string[]>(() => normalizeFilePath(props.filePath))

async function refreshDirData(basePath: string[]) {
    const data = await get_tree_data(basePath)
    const tree_data = new TreeData(data!)

    if (tree_data.is_dir()) {
        is_dir.value = true
        is_file.value = false
        const dirData = tree_data.dir_data()!
        const files = dirData.files().map((f) => f.name())
        const dirs = dirData.dirs().map((d) => d.name())
        dir_data.value = new DirData(dirs, files)
    }
    if (tree_data.is_file()) {
        is_dir.value = false
        is_file.value = true
        const fileData = tree_data.file_data()!
        code.value = fileData.lines()
        console.log(code.value)
    }


}



watch(normalizedPath, async (newPath, oldPath) => {
    if (oldPath && isSamePath(newPath, oldPath)) {
        return
    }

    await refreshDirData(newPath)
}, { immediate: true })

</script>

<script lang="ts">
async function get_tree_data(path: string[]) {
    let path_str = path.join("/");
    let path_index = new PathIndex(path_str);
    let data = await fetchFileData(path_index.path_str("lsp-index") + "/tree.data");
    return data
}


class DirData {
    readonly dirs: string[]
    readonly files: string[]
    constructor(dirs: string[], files: string[]) {
        this.dirs = dirs
        this.files = files
    }

}
</script>



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
