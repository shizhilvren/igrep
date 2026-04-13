<template>
    <div ref="el" class="monaco-container"></div>
</template>


<script lang="ts">

</script>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import loader from '@monaco-editor/loader'
import * as monaco from 'monaco-editor'
import { registerHoverProvider } from '@/components/lsp/hoverProvider'
import { registerDefinitionProvider } from '@/components/lsp/definitionProvider'
import { applySemanticHighlight } from '@/components/lsp/semanticHighlighter'
import { FileContent, SemanticTokens, HoverData, DefinitionData, Files } from '@/components/lsp/file'

const el = ref<HTMLElement | null>(null)
const addModelPromise = ref<Map<string, Promise<void>>>(new Map())

let editor: monaco.editor.IStandaloneCodeEditor | null = null

let sizeDispose: monaco.IDisposable | null = null
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let hoverDispose: monaco.IDisposable | null = null
let definitionDispose: monaco.IDisposable | null = null
let openerDispose: monaco.IDisposable | null = null
const createdModelUris = new Set<string>()

const emit = defineEmits<{
    'addFileToModel': [file_path: string]
    'changeFile': [file_path: string]
}>()


const props = defineProps<{
    filePath: string[]
    files: Files
}>()



const file_uri = computed(() => {
    return toModelUri(props.filePath)
})

const semantic = computed(() => {
    return props.files.getFileContent(props.filePath)?.semanticTokens
})

const code = computed(() => {
    const file = props.files.getFileContent(props.filePath)
    return { "code": file?.code, "language": file?.language, "uri": file_uri }
})



const hovers = computed(() => {
    return new Map([...props.files.files.entries()].map(([name, file]) => {
        return ["file://" + name, file.hoverData]
    }))
})

const defitition = computed(() => {
    return new Map([...props.files.files.entries()].map(([name, file]) => {
        return ["file://" + name, file.definitionData]
    }))
})

function addFileToModel(file_path: string) {
    const file_path_array = file_path.split('/').filter((e) => { return e != "" })
    const have_file = !!props.files.getFileContent(file_path_array);
    if (have_file) {
        return null
    } else {
        const wait = new Promise<void>((resolve, rejects) => {
            const stop = watch(
                () => props.files.getFileContent(file_path_array),
                (_, new_model) => {
                    const uri = monaco.Uri.parse("file://" + file_path)
                    const modelText = new_model?.code.join('\n')
                    if (!monaco.editor.getModel(uri) && modelText) {
                        monaco.editor.createModel(modelText, "cpp", uri)
                        stop()
                        resolve()
                    }
                },
                { deep: true },

            )
        })
        emit("addFileToModel", file_path)
        return wait
    }

}


function updateSemanticHighlight(semantic_tokens: SemanticTokens | undefined) {
    decorations = applySemanticHighlight(editor, semantic_tokens, decorations)
}

function updateHoverProvider() {
    hoverDispose?.dispose()
    hoverDispose = registerHoverProvider("cpp", hovers)
}

function updateDefinitionProvider() {
    definitionDispose?.dispose()
    definitionDispose = registerDefinitionProvider("cpp", defitition, addFileToModel)
}

function toEditorTargetRange(selectionOrPosition?: monaco.IRange | monaco.IPosition): monaco.Range | undefined {
    if (!selectionOrPosition) {
        return undefined
    }

    const maybeRange = selectionOrPosition as monaco.IRange
    if (
        typeof maybeRange.startLineNumber === 'number'
        && typeof maybeRange.startColumn === 'number'
        && typeof maybeRange.endLineNumber === 'number'
        && typeof maybeRange.endColumn === 'number'
    ) {
        return new monaco.Range(
            maybeRange.startLineNumber,
            maybeRange.startColumn,
            maybeRange.endLineNumber,
            maybeRange.endColumn,
        )
    }

    const maybePosition = selectionOrPosition as monaco.IPosition
    if (typeof maybePosition.lineNumber === 'number' && typeof maybePosition.column === 'number') {
        return new monaco.Range(
            maybePosition.lineNumber,
            maybePosition.column,
            maybePosition.lineNumber,
            maybePosition.column,
        )
    }

    return undefined
}

function expandCollapsedRangeToWord(model: monaco.editor.ITextModel, range: monaco.Range): monaco.Range {
    if (!range.isEmpty()) {
        return range
    }

    const word = model.getWordAtPosition({
        lineNumber: range.startLineNumber,
        column: range.startColumn,
    })
    console.log(word, range)

    if (!word) {
        return range
    }

    return new monaco.Range(
        range.startLineNumber,
        word.startColumn,
        range.startLineNumber,
        word.endColumn,
    )
}

function toModelUri(filePath: string[] | undefined): monaco.Uri {
    const normalizedPath = (filePath ?? [])
        .flatMap((part) => part.split('/'))
        .map((part) => part.trim())
        .filter((part) => part.length > 0)
        .join('/')

    if (normalizedPath.length === 0) {
        return monaco.Uri.parse('inmemory://igrep/untitled')
    }

    return monaco.Uri.parse(`file:///${normalizedPath}`)
}

function ensureFileModel(code: string[], language: string, file_uri: monaco.Uri) {

    if (!editor) {
        return
    }

    const modelText = code.join('\n')

    let model = monaco.editor.getModel(file_uri)
    if (!model) {
        model = monaco.editor.createModel(modelText, language, file_uri)
        createdModelUris.add(file_uri.toString())
    } else {
        if (model.getValue() !== modelText) {
            model.setValue(modelText)
        }
        monaco.editor.setModelLanguage(model, language)
    }

    if (editor.getModel() !== model) {
        editor.setModel(model)
    }
}

onMounted(async () => {
    loader.config({ monaco })
    await loader.init()

    if (!el.value) {
        return
    }

    editor = monaco.editor.create(el.value, {
        value: '',
        // language: props.language,
        theme: 'vs',
        readOnly: true,
        automaticLayout: true,
        wordWrap: 'off',
    })

    openerDispose = monaco.editor.registerEditorOpener({
        openCodeEditor: (_, resource, selectionOrPosition) => {
            if (!editor) {
                return false
            }
            const model = monaco.editor.getModel(resource)
            if (!model) {
                return false
            }

            editor.setModel(model)
            const targetRange = toEditorTargetRange(selectionOrPosition)
            if (targetRange) {
                const highlightRange = expandCollapsedRangeToWord(model, targetRange)
                // console.log('Highlight range:', highlightRange)
                editor.setSelection(highlightRange)
                editor.revealRangeInCenter(highlightRange)
            }

            emit("changeFile", resource.path)
            // createdModelUris.add(resource.toString())



            editor.focus()
            return true
        },
    })

    const initialModel = editor.getModel()
    if (code.value.code && code.value.language) {
        ensureFileModel(code.value.code, code.value.language, code.value.uri.value)
    }
    if (initialModel && initialModel !== editor.getModel()) {
        initialModel.dispose()
    }

    updateHoverProvider()
    updateSemanticHighlight(semantic.value)
    updateDefinitionProvider()
})




// watch(
//     () => props.language,
//     () => {
//         updateHoverProvider()
//         updateDefinitionProvider()
//     },
// )

// watch(
//     () => props.hoverData,
//     () => {
//         updateHoverProvider()
//     },
// )

// watch(
//     () => props.definitionData,
//     () => {
//         updateDefinitionProvider()
//     },
// )

watch(
    () => semantic.value,
    (_, new_semantic_tokens) => {
        updateSemanticHighlight(new_semantic_tokens)
    },
    { deep: true },
)

watch(
    () => code,
    (_, new_val) => {
        if (!editor || !new_val.value.code || !new_val.value.language) {
            return
        }
        ensureFileModel(new_val.value.code, new_val.value.language, new_val.value.uri.value)
        // updateDefinitionProvider()
        // updateSemanticHighlight()
    },
    { deep: true },
)

onBeforeUnmount(() => {
    hoverDispose?.dispose()
    hoverDispose = null
    definitionDispose?.dispose()
    definitionDispose = null
    openerDispose?.dispose()
    openerDispose = null
    sizeDispose?.dispose()
    sizeDispose = null
    decorations?.clear()
    decorations = null

    for (const uriStr of createdModelUris) {
        const model = monaco.editor.getModel(monaco.Uri.parse(uriStr))
        model?.dispose()
    }
    createdModelUris.clear()

    editor?.dispose()
    editor = null
})
</script>



<style scoped>
@import './semanticTokens.css';

.monaco-container {
    width: 100%;
    height: 100%;
}

/* Editor behavior */
:deep(.monaco-editor .cursor) {
    display: none !important;
}

/* :deep(.monaco-editor .view-overlays .current-line) {
    border: 0 !important;
} */
</style>