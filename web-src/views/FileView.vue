<template>
    <main class="file-view">
        <div style="padding-left: 10px;">
            <FilePathBar v-bind="{ filePath: path }" />
        </div>
        <DirTree v-bind="{
            dirs: dir_data.dirs,
            files: dir_data.files,
            base_path: path
        }" />
        <div class="file-editor">
            <OneFile v-bind="{ language: 'rust', code: code }" />
        </div>
    </main>
</template>

<script setup lang="ts">
import FilePathBar from '@/components/lsp/FilePathBar.vue';
import OneFile from '@/components/lsp/OneFile.vue';
import { languages } from 'monaco-editor';
import { onMounted, ref, toRef, watch } from 'vue';
import { PathIndex, TreeData } from 'igrep';
import { fetchFileData } from "@/utils/utils"
import DirTree from '@/components/lsp/DirTree.vue';

const props = defineProps<{
    filePath: string | string[]
}>()

const path = ref<string[]>([])
const dir_data = ref<DirData>(new DirData([], []))
function update_base_path() {
    console.log("some", props.filePath, "some")
    if (props.filePath == "") {
        path.value = []
    } else {
        path.value = Array.isArray(props.filePath) ? props.filePath : [props.filePath]

    }
}
onMounted(async () => {
    console.log(Array.isArray(props.filePath))
    update_base_path();
    let data = await get_tree_data(path.value);
    let tree_data = new TreeData(data!);
    console.log("file data", tree_data.is_dir(), tree_data.is_file());
    if (tree_data.is_dir()) {
        let data = tree_data.dir_data()!;
        let files = data.files().map((f) => f.name());
        let dirs = data.dirs().map((d) => d.name());
        console.log("files", files, "dirs", dirs);
        dir_data.value = new DirData(dirs, files);
    }
})

let code = [
    "fn main() {",
    "    let name = \"igrep\";",
    "    println!(\"hello {}\", name);",
    "}"
]

watch(() => props.filePath, async (new_path) => {
    console.log(`x is ${new_path}`)
    update_base_path();
    let data = await get_tree_data(path.value);
    let tree_data = new TreeData(data!);
    console.log("file data", tree_data.is_dir(), tree_data.is_file());
    if (tree_data.is_dir()) {
        let data = tree_data.dir_data()!;
        let files = data.files().map((f) => f.name());
        let dirs = data.dirs().map((d) => d.name());
        console.log("files", files, "dirs", dirs);
        dir_data.value = new DirData(dirs, files);

    }
})

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
