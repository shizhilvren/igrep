<template>
    <div ref="el" class="monaco-container"></div>
</template>


<script lang="ts">
export class SemanticToken {
    readonly delta_line: number
    readonly delta_start: number
    readonly length: number
    readonly token_type: number
    readonly token_modifiers_bitset: number
    constructor(delta_line: number, delta_start: number, length: number, token_type: number, token_modifiers_bitset: number) {
        this.delta_line = delta_line
        this.delta_start = delta_start
        this.length = length
        this.token_type = token_type
        this.token_modifiers_bitset = token_modifiers_bitset
    }
}
export class SemanticTokens {
    readonly data: SemanticToken[]
    constructor(data: SemanticToken[]) {
        this.data = data
    }
}

export type HoverPosition = {
    line: number
    character: number
}

export class HoverDataModel {
    readonly start: HoverPosition
    readonly end: HoverPosition
    readonly hover: string

    constructor(start: HoverPosition, end: HoverPosition, hover: string) {
        this.start = start
        this.end = end
        this.hover = hover
    }
}

export class DefinitionLocationModel {
    readonly fileName: string
    readonly start: HoverPosition
    readonly end: HoverPosition

    constructor(fileName: string, start: HoverPosition, end: HoverPosition) {
        this.fileName = fileName
        this.start = start
        this.end = end
    }
}

export class DefinitionDataModel {
    readonly start: HoverPosition
    readonly end: HoverPosition
    readonly locations: DefinitionLocationModel[]

    constructor(start: HoverPosition, end: HoverPosition, locations: DefinitionLocationModel[]) {
        this.start = start
        this.end = end
        this.locations = locations
    }
}
</script>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import loader from '@monaco-editor/loader'
import * as monaco from 'monaco-editor'
import { registerHoverProvider } from '@/components/lsp/hoverProvider'
import { registerDefinitionProvider } from '@/components/lsp/definitionProvider'
import { applySemanticHighlight } from '@/components/lsp/semanticHighlighter'

const el = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

let sizeDispose: monaco.IDisposable | null = null
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let hoverDispose: monaco.IDisposable | null = null
let definitionDispose: monaco.IDisposable | null = null
let openerDispose: monaco.IDisposable | null = null
const createdModelUris = new Set<string>()

const props = defineProps<{
    code: string[]
    language: string
    semanticTokens?: SemanticTokens
    hoverData?: HoverDataModel[]
    definitionData?: DefinitionDataModel[]
    filePath?: string[]
}>()

function updateSemanticHighlight() {
    decorations = applySemanticHighlight(editor, props.semanticTokens, decorations)
}

function updateHoverProvider() {
    hoverDispose?.dispose()
    hoverDispose = registerHoverProvider(props.language, props.hoverData)
}

function updateDefinitionProvider() {
    definitionDispose?.dispose()
    definitionDispose = registerDefinitionProvider(props.language, props.definitionData)
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

function ensureFileModel() {
    if (!editor) {
        return
    }

    const uri = toModelUri(props.filePath)
    const modelText = props.code.join('\n')

    let model = monaco.editor.getModel(uri)
    if (!model) {
        model = monaco.editor.createModel(modelText, props.language, uri)
        createdModelUris.add(uri.toString())
    } else {
        if (model.getValue() !== modelText) {
            model.setValue(modelText)
        }
        monaco.editor.setModelLanguage(model, props.language)
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
        language: props.language,
        theme: 'vs',
        readOnly: true,
        automaticLayout: true,
        wordWrap: 'off',
    })

    openerDispose = monaco.editor.registerEditorOpener({
        async openCodeEditor(_, resource, selectionOrPosition) {
            if (!editor) {
                return false
            }

            const model = monaco.editor.getModel(resource)
            if (!model) {
                return false
            }

            editor.setModel(model)
            createdModelUris.add(resource.toString())

            const targetRange = toEditorTargetRange(selectionOrPosition)
            if (targetRange) {
                const highlightRange = expandCollapsedRangeToWord(model, targetRange)
                console.log('Highlight range:', highlightRange)
                editor.setSelection(highlightRange)
                editor.revealRangeInCenter(highlightRange)
            }

            editor.focus()
            return true
        },
    })

    const initialModel = editor.getModel()
    ensureFileModel()
    if (initialModel && initialModel !== editor.getModel()) {
        initialModel.dispose()
    }

    updateHoverProvider()
    updateDefinitionProvider()
    updateSemanticHighlight()
})

watch(
    () => [props.code, props.language, props.filePath],
    () => {
        if (!editor) return

        ensureFileModel()
        updateDefinitionProvider()
        updateSemanticHighlight()
    },
    { deep: true },
)

watch(
    () => props.semanticTokens,
    () => {
        updateSemanticHighlight()
    },
    { deep: true },
)

watch(
    () => props.language,
    () => {
        updateHoverProvider()
        updateDefinitionProvider()
    },
)

watch(
    () => props.hoverData,
    () => {
        updateHoverProvider()
    },
)

watch(
    () => props.definitionData,
    () => {
        updateDefinitionProvider()
    },
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