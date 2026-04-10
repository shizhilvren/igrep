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

onMounted(async () => {
    loader.config({ monaco })
    await loader.init()

    if (!el.value) {
        return
    }

    editor = monaco.editor.create(el.value, {
        value: props.code.join('\n'),
        language: props.language,
        theme: 'vs',
        readOnly: true,
        automaticLayout: true,
        wordWrap: 'off',
    })

    updateHoverProvider()
    updateDefinitionProvider()
    updateSemanticHighlight()
})

watch(
    () => [props.code, props.language],
    () => {
        if (!editor) return
        const model = editor.getModel()
        if (!model) return

        model.setValue(props.code.join('\n'))
        monaco.editor.setModelLanguage(model, props.language)
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
    sizeDispose?.dispose()
    sizeDispose = null
    decorations?.clear()
    decorations = null
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