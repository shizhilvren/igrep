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
</script>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import loader from '@monaco-editor/loader'
import * as monaco from 'monaco-editor'

const el = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

let sizeDispose: monaco.IDisposable | null = null
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let hoverDispose: monaco.IDisposable | null = null

const props = defineProps<{
    code: string[]
    language: string
    semanticTokens?: SemanticTokens
}>()

const tokenTypeClassMap = [
    'st-file',
    'st-module',
    'st-namespace',
    'st-package',
    'st-class',
    'st-method',
    'st-property',
    'st-field',
    'st-constructor',
    'st-enum',
    'st-interface',
    'st-function',
    'st-variable',
    'st-constant',
    'st-string',
    'st-number',
    'st-boolean',
    'st-array',
    'st-object',
    'st-key',
    'st-null',
    'st-enum-member',
    'st-struct',
    'st-event',
    'st-operator',
    'st-type-parameter',

    // Keep semantic-token-specific values for compatibility with existing back-end indexes.
    'st-type',
    'st-parameter',
    'st-macro',
    'st-keyword',
    'st-modifier',
    'st-comment',
    'st-regexp',
    'st-decorator',
]

function updateSemanticHighlight() {
    if (!editor) return
    const model = editor.getModel()
    if (!model) return

    const tokens = props.semanticTokens ?? { data: [] }
    if (tokens.data.length === 0) {
        decorations?.set([])
        return
    }

    const next: monaco.editor.IModelDeltaDecoration[] = []
    let line = 0
    let start = 0

    for (const token of tokens.data) {
        line += token.delta_line
        if (token.delta_line === 0) {
            start += token.delta_start
        } else {
            start = token.delta_start
        }

        const lineNumber = line + 1
        if (lineNumber < 1 || lineNumber > model.getLineCount()) {
            continue
        }

        const startColumn = start + 1
        const endColumn = startColumn + token.length
        const lineMaxColumn = model.getLineMaxColumn(lineNumber)
        const safeStart = Math.max(1, Math.min(startColumn, lineMaxColumn))
        const safeEnd = Math.max(safeStart, Math.min(endColumn, lineMaxColumn))
        const typeClass = tokenTypeClassMap[token.token_type] ?? 'st-default'

        next.push({
            range: new monaco.Range(lineNumber, safeStart, lineNumber, safeEnd),
            options: { inlineClassName: `semantic-token ${typeClass}` },
        })
    }

    if (!decorations) {
        decorations = editor.createDecorationsCollection(next)
        return
    }
    decorations.set(next)
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

onBeforeUnmount(() => {
    hoverDispose?.dispose()
    hoverDispose = null
    sizeDispose?.dispose()
    sizeDispose = null
    decorations?.clear()
    decorations = null
    editor?.dispose()
    editor = null
})
</script>



<style scoped>
.monaco-container {
    width: 100%;
    height: 100%;
}

:deep(.my-string-highlight) {
    background: rgba(255, 220, 120, 0.45);
    border-radius: 2px;
}

:deep(.semantic-token.st-namespace) {
    color: #0b57d0;
}

:deep(.semantic-token.st-file) {
    color: #334155;
}

:deep(.semantic-token.st-module) {
    color: #1d4ed8;
}

:deep(.semantic-token.st-package) {
    color: #1e40af;
}

:deep(.semantic-token.st-type) {
    color: #00639b;
}

:deep(.semantic-token.st-class) {
    color: #0f766e;
}

:deep(.semantic-token.st-enum) {
    color: #7c3aed;
}

:deep(.semantic-token.st-interface) {
    color: #155e75;
}

:deep(.semantic-token.st-struct) {
    color: #0d9488;
}

:deep(.semantic-token.st-type-parameter) {
    color: #c2410c;
}

:deep(.semantic-token.st-parameter) {
    color: #9a3412;
}

:deep(.semantic-token.st-variable) {
    color: #1f2937;
}

:deep(.semantic-token.st-property) {
    color: #2563eb;
}

:deep(.semantic-token.st-field) {
    color: #2563eb;
}

:deep(.semantic-token.st-constructor) {
    color: #0f766e;
}

:deep(.semantic-token.st-enum-member) {
    color: #7c2d12;
}

:deep(.semantic-token.st-event) {
    color: #7c3aed;
}

:deep(.semantic-token.st-function) {
    color: #0d9488;
}

:deep(.semantic-token.st-method) {
    color: #0f766e;
}

:deep(.semantic-token.st-macro) {
    color: #be123c;
}

:deep(.semantic-token.st-keyword) {
    color: #b91c1c;
    font-weight: 600;
}

:deep(.semantic-token.st-modifier) {
    color: #6b7280;
}

:deep(.semantic-token.st-comment) {
    color: #6b7280;
    font-style: italic;
}

:deep(.semantic-token.st-string) {
    color: #b45309;
}

:deep(.semantic-token.st-number) {
    color: #9333ea;
}

:deep(.semantic-token.st-constant) {
    color: #7c3aed;
    font-weight: 600;
}

:deep(.semantic-token.st-boolean) {
    color: #7e22ce;
    font-weight: 600;
}

:deep(.semantic-token.st-array) {
    color: #0f766e;
}

:deep(.semantic-token.st-object) {
    color: #0f766e;
}

:deep(.semantic-token.st-key) {
    color: #1d4ed8;
}

:deep(.semantic-token.st-null) {
    color: #6b7280;
    font-style: italic;
}

:deep(.semantic-token.st-regexp) {
    color: #d97706;
}

:deep(.semantic-token.st-operator) {
    color: #374151;
}

:deep(.semantic-token.st-decorator) {
    color: #c026d3;
}

:deep(.semantic-token.st-default) {
    color: #1f2937;
}



:deep(.monaco-editor .cursor) {
    display: none !important;
}

/* :deep(.monaco-editor .view-overlays .current-line) {
    border: 0 !important;
} */
</style>