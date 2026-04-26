<template>
    <main class="file-view">
        <div style="padding-left: 10px;">
            <FilePathBar v-bind="{ filePath: normalizedPath }" />
        </div>
        <!-- <el-button plain @click="open2">Info</el-button> -->
        <div v-loading="loading" :class="['content-wrap', is_file ? 'content-wrap--fill' : 'content-wrap--auto']">
            <DirTree v-if="is_dir" class="dir-content" v-bind="{
                dirs: dir_data.dirs,
                files: dir_data.files,
                base_path: normalizedPath
            }" />
            <OneFile v-if="is_file" class="file-editor" v-bind="{
                files: files,
                filePath: normalizedPath,
            }" @add-file-to-model="addFileToModel" @change-file="changeFile" @hover="hover" @did-open="didOpen"
                @did-close="didClose" />
        </div>
    </main>
</template>

<script setup lang="ts">
import { DefinitionData, DefinitionLocationModel, FileContent, Files, HoverData, ReferenceData, ReferenceLocationModel, SemanticToken, SemanticTokens } from '@/components/lsp/file'
import FilePathBar from '@/components/lsp/FilePathBar.vue';
import OneFile from '@/components/lsp/OneFile.vue';
import { computed, onMounted, ref, toRef, watch } from 'vue';
import { useRouter } from 'vue-router';
import * as igrep from 'igrep';
import { fetchFileData } from "@/utils/utils"
import DirTree from '@/components/lsp/DirTree.vue';
import { startVMandlisten } from '@/utils/clang_lsp';
import type { V86 } from '../../v86/v86';
import type { LSPClient } from '@/utils/lsp_client';

const router = useRouter();

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
const loading = ref(false)
const files = ref<Files>(new Files())
const vm = ref<{
    isReady: boolean;
    instance: V86;
} | undefined>(undefined)

const client = ref<LSPClient | undefined>(undefined)

const VMStartDone = computed(() => {
    return client.value !== undefined
})


onMounted(async () => {
    const ans = await startVMandlisten();
    vm.value = { instance: ans.instance, isReady: ans.isReady }
    client.value = ans.client
    console.log("lsp server start", vm.value.isReady);
});

async function waitVMStart() {
    const wait = new Promise((resolve: (_: void) => void) => {
        if (VMStartDone.value) {
            resolve()
        } else {
            const stop = watch(
                () => VMStartDone.value,
                (_) => {
                    resolve()
                },
                { deep: true },
            )
        }
    })
    return wait
}

function changeFile(file_path: string) {
    console.log("change file", file_path)
    const file_path_array = file_path.split('/').filter((e) => {
        return e != ''
    })
    router.push({ name: "files", params: { filePath: file_path_array } })
    // await refreshDirData(file_path_array)
}

async function hover(file_path: string, line: number, character: number, resolve: (val: unknown) => void) {
    await waitVMStart()
    client.value?.hover(file_path, line, character, resolve)
}

async function didOpen(file_path: string, content: string[]) {
    await waitVMStart()
    client.value?.didOpen(file_path, content)
}

async function didClose(file_path: string) {
    await waitVMStart()
    client.value?.didClose(file_path)
}

async function addFileToModel(file_path: string) {
    console.log("add file to model", file_path)
    const file_path_array = file_path.split('/').filter((e) => {
        return e != ''
    })
    let data = await get_tree_data(file_path_array)
    if (data) {
        try {
            const tree_data = new igrep.TreeData(data)
            if (tree_data.is_file()) {
                const fileData = tree_data.file_data()!
                const semanticTokens = fileData.semantic_tokens() ? new SemanticTokens(fileData.semantic_tokens()!.map(
                    (t) => new SemanticToken(t.delta_line(), t.delta_start(), t.length(), t.token_type(), t.token_modifiers_bitset()))) : undefined
                let file = new FileContent(file_path_array, fileData.lines(), "cpp", semanticTokens!)
                files.value.addFileContent(file_path_array, file)
                console.log("files", files)

                // const hoverRawData = await get_hover_data(file_path_array)
                // const hovers = parseHoverData(hoverRawData)
                // const definitionRawData = await get_definition_data(file_path_array)
                // const defititions = parseDefinitionData(definitionRawData)
                // const referencesRawData = await get_references_data(file_path_array)
                // const references = parseReferencesData(referencesRawData)
                // files.value.getFileContent(file_path_array)?.setHoverData(hovers)
                // files.value.getFileContent(file_path_array)?.setDefinitionData(defititions)
                // files.value.getFileContent(file_path_array)?.setReferenceData(references)
            }
        } catch (e) {
            let file = new FileContent(file_path_array, ["this file not in index", file_path], "cpp", new SemanticTokens([]))
            files.value.addFileContent(file_path_array, file)
            console.log("add empty file", file_path, file)
            return 1
        }
    }
    return 1
}

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
    loading.value = true
    try {
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

            loading.value = false

            // const hoverRawData = await get_hover_data(basePath)
            // const hovers = parseHoverData(hoverRawData)
            // const definitionRawData = await get_definition_data(basePath)
            // const defititions = parseDefinitionData(definitionRawData)
            // const referencesRawData = await get_references_data(basePath)
            // const references = parseReferencesData(referencesRawData)
            // files.value.getFileContent(basePath)?.setHoverData(hovers)
            // files.value.getFileContent(basePath)?.setDefinitionData(defititions)
            // files.value.getFileContent(basePath)?.setReferenceData(references)
            // console.log(code.value)
        }
    } catch (e) {
        console.error("Failed to add file to model:", e)
    } finally {
        loading.value = false
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
function parseDefinitionData(data: Uint8Array | undefined): DefinitionData[] {
    if (!data) {
        return []
    }

    let definitions = new igrep.DefinitionsData(data).definitions().map((definition) => {
        const range = definition.range()
        const start = range.start()
        const end = range.end()
        const locations = definition.locations().map((location) => {
            const locationRange = location.range()
            const locationStart = locationRange.start()
            const locationEnd = locationRange.end()

            return new DefinitionLocationModel(
                location.file_name(),
                {
                    line: locationStart.line(),
                    character: locationStart.character(),
                },
                {
                    line: locationEnd.line(),
                    character: locationEnd.character(),
                },
            )
        })

        return new DefinitionData(
            {
                line: start.line(),
                character: start.character(),
            },
            {
                line: end.line(),
                character: end.character(),
            },
            locations,
        )
    });
    // console.log("this is definitions", definitions);
    return definitions
}

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

function parseReferencesData(data: Uint8Array | undefined): ReferenceData[] {
    if (!data) {
        return []
    }

    return new igrep.ReferencesData(data).references().map((reference) => {
        const range = reference.range()
        const start = range.start()
        const end = range.end()
        const locations = reference.locations().map((location) => {
            const locationRange = location.range()
            const locationStart = locationRange.start()
            const locationEnd = locationRange.end()

            return new ReferenceLocationModel(
                location.file_name(),
                {
                    line: locationStart.line(),
                    character: locationStart.character(),
                },
                {
                    line: locationEnd.line(),
                    character: locationEnd.character(),
                },
            )
        })

        return new ReferenceData(
            {
                line: start.line(),
                character: start.character(),
            },
            {
                line: end.line(),
                character: end.character(),
            },
            locations,
        )
    })
}

async function get_tree_data(path: string[]) {
    let path_str = path.join("/");
    let path_index = new igrep.PathIndex(path_str);
    let data = await fetchFileData(path_index.path_str("lsp-index") + "/tree.data");
    return data
}

// async function get_hover_data(path: string[]) {
//     let path_str = path.join("/");
//     let path_index = new igrep.PathIndex(path_str);
//     let data = await fetchFileData(path_index.path_str("lsp-index") + "/hover.data");
//     return data
// }

// async function get_definition_data(path: string[]) {
//     let path_str = path.join("/");
//     let path_index = new igrep.PathIndex(path_str);
//     let data = await fetchFileData(path_index.path_str("lsp-index") + "/definition.data");
//     return data
// }

// async function get_references_data(path: string[]) {
//     let path_str = path.join("/");
//     let path_index = new igrep.PathIndex(path_str);
//     let data = await fetchFileData(path_index.path_str("lsp-index") + "/references.data");
//     return data
// }

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

.content-wrap {
    width: 100%;
}

.content-wrap--auto {
    height: fit-content;
}

.content-wrap--fill {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
}

.dir-content {
    height: fit-content;
}
</style>
