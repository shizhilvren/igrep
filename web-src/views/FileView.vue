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
        <OneFile v-if="is_file" class="file-editor" v-bind="{
            files: files,
            filePath: normalizedPath
        }" />
    </main>
</template>

<script setup lang="ts">
import { FileContent, Files, HoverData, SemanticToken, SemanticTokens } from '@/components/lsp/file'
import FilePathBar from '@/components/lsp/FilePathBar.vue';
import OneFile from '@/components/lsp/OneFile.vue';
import { computed, ref, watch } from 'vue';
import * as igrep from 'igrep';
import { fetchFileData } from "@/utils/utils"
import DirTree from '@/components/lsp/DirTree.vue';

const props = defineProps<{
    filePath: string | string[]
}>()

const dir_data = ref<DirData>(new DirData([], []))
// const code = ref<string[]>([])
// const semanticTokens = ref<SemanticTokens | undefined>(undefined)
// const hoverData = ref<HoverDataModel[]>([])
// const definitionData = ref<DefinitionDataModel[]>([])
const is_dir = ref(false)
const is_file = ref(false)
const files = ref<Files>(new Files())



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
    const tree_data = new igrep.TreeData(data!)


    if (tree_data.is_dir()) {
        is_dir.value = true
        is_file.value = false
        // hoverData.value = []
        // definitionData.value = []
        const dirData = tree_data.dir_data()!
        const files = dirData.files().map((f) => f.name())
        const dirs = dirData.dirs().map((d) => d.name())
        dir_data.value = new DirData(dirs, files)
    } else if (tree_data.is_file()) {
        is_dir.value = false
        is_file.value = true
        const fileData = tree_data.file_data()!
        const semanticTokens = fileData.semantic_tokens() ? new SemanticTokens(fileData.semantic_tokens()!.map(
            (t) => new SemanticToken(t.delta_line(), t.delta_start(), t.length(), t.token_type(), t.token_modifiers_bitset()))) : undefined
        let file = new FileContent(basePath, fileData.lines(), "cpp", semanticTokens!)
        files.value.addFileContent(basePath, file)
        console.log("files", files)

        const hoverRawData = await get_hover_data(basePath)
        const definitionRawData = await get_definition_data(basePath)
        const hovers = parseHoverData(hoverRawData)
        files.value.getFileContent(basePath)?.setHoverData(hovers)
        // definitionData.value = parseDefinitionData(definitionRawData)
        // console.log(code.value)
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
// function parseDefinitionData(data: Uint8Array | undefined): DefinitionDataModel[] {
//     if (!data) {
//         return []
//     }

//     let definitions = new DefinitionsData(data).definitions().map((definition) => {
//         const range = definition.range()
//         const start = range.start()
//         const end = range.end()
//         const locations = definition.locations().map((location) => {
//             const locationRange = location.range()
//             const locationStart = locationRange.start()
//             const locationEnd = locationRange.end()

//             return new DefinitionLocationModel(
//                 location.file_name(),
//                 {
//                     line: locationStart.line(),
//                     character: locationStart.character(),
//                 },
//                 {
//                     line: locationEnd.line(),
//                     character: locationEnd.character(),
//                 },
//             )
//         })

//         return new DefinitionDataModel(
//             {
//                 line: start.line(),
//                 character: start.character(),
//             },
//             {
//                 line: end.line(),
//                 character: end.character(),
//             },
//             locations,
//         )
//     });
//     console.log("this is definitions", definitions);
//     return definitions
// }

function parseHoverData(data: Uint8Array | undefined): HoverData[] {
    if (!data) {
        return []
    }

    return new igrep.HoversData(data).hovers().map((hover) => {
        const range = hover.range()
        const start = range.start()
        const end = range.end()
        const hoverText = hover.hover()

        return new HoverData(
            {
                line: start.line(),
                character: start.character(),
            },
            {
                line: end.line(),
                character: end.character(),
            },
            hoverText,
        )
    })
}

async function get_tree_data(path: string[]) {
    let path_str = path.join("/");
    let path_index = new igrep.PathIndex(path_str);
    let data = await fetchFileData(path_index.path_str("lsp-index") + "/tree.data");
    return data
}

async function get_hover_data(path: string[]) {
    let path_str = path.join("/");
    let path_index = new igrep.PathIndex(path_str);
    let data = await fetchFileData(path_index.path_str("lsp-index") + "/hover.data");
    return data
}

async function get_definition_data(path: string[]) {
    let path_str = path.join("/");
    let path_index = new igrep.PathIndex(path_str);
    let data = await fetchFileData(path_index.path_str("lsp-index") + "/definition.data");
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
