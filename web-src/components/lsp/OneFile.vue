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

const el = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

let sizeDispose: monaco.IDisposable | null = null
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let hoverDispose: monaco.IDisposable | null = null
let definitionDispose: monaco.IDisposable | null = null

type SortedHoverItem = {
    startLine: number
    startChar: number
    endLine: number
    endChar: number
    hover: string
}

type SortedRangeItem = {
    startLine: number
    startChar: number
    endLine: number
    endChar: number
}

type SortedDefinitionItem = {
    startLine: number
    startChar: number
    endLine: number
    endChar: number
    locations: {
        fileName: string
        startLine: number
        startChar: number
        endLine: number
        endChar: number
    }[]
}

let sortedHoverItems: SortedHoverItem[] = []
let sortedHoverSource: HoverDataModel[] | undefined = undefined
let sortedDefinitionItems: SortedDefinitionItem[] = []
let sortedDefinitionSource: DefinitionDataModel[] | undefined = undefined

const props = defineProps<{
    code: string[]
    language: string
    semanticTokens?: SemanticTokens
    hoverData?: HoverDataModel[]
    definitionData?: DefinitionDataModel[]
    filePath?: string[]
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

function isPositionInRange(position: monaco.Position, range: monaco.Range): boolean {
    const line = position.lineNumber
    const column = position.column
    const startsAfter = line > range.startLineNumber || (line === range.startLineNumber && column >= range.startColumn)
    const endsBefore = line < range.endLineNumber || (line === range.endLineNumber && column <= range.endColumn)
    return startsAfter && endsBefore
}

function comparePosition(aLine: number, aChar: number, bLine: number, bChar: number): number {
    if (aLine !== bLine) {
        return aLine - bLine
    }
    return aChar - bChar
}

function buildSortedHoverItems(hovers: HoverDataModel[] | undefined): SortedHoverItem[] {
    if (!hovers || hovers.length === 0) {
        return []
    }

    return hovers
        .map((hover) => ({
            startLine: hover.start.line + 1,
            startChar: hover.start.character + 1,
            endLine: hover.end.line + 1,
            endChar: hover.end.character + 1,
            hover: hover.hover,
        }))
        .sort((a, b) => {
            const startCmp = comparePosition(a.startLine, a.startChar, b.startLine, b.startChar)
            if (startCmp !== 0) {
                return startCmp
            }
            return comparePosition(a.endLine, a.endChar, b.endLine, b.endChar)
        })
}

function buildSortedDefinitionItems(definitions: DefinitionDataModel[] | undefined): SortedDefinitionItem[] {
    if (!definitions || definitions.length === 0) {
        return []
    }

    return definitions
        .map((definition) => ({
            startLine: definition.start.line + 1,
            startChar: definition.start.character + 1,
            endLine: definition.end.line + 1,
            endChar: definition.end.character + 1,
            locations: definition.locations.map((location) => ({
                fileName: location.fileName,
                startLine: location.start.line + 1,
                startChar: location.start.character + 1,
                endLine: location.end.line + 1,
                endChar: location.end.character + 1,
            })),
        }))
        .sort((a, b) => {
            const startCmp = comparePosition(a.startLine, a.startChar, b.startLine, b.startChar)
            if (startCmp !== 0) {
                return startCmp
            }
            return comparePosition(a.endLine, a.endChar, b.endLine, b.endChar)
        })
}

function ensureSortedHoverItems() {
    if (sortedHoverSource === props.hoverData) {
        return
    }
    sortedHoverSource = props.hoverData
    sortedHoverItems = buildSortedHoverItems(props.hoverData)
}

function ensureSortedDefinitionItems() {
    if (sortedDefinitionSource === props.definitionData) {
        return
    }
    sortedDefinitionSource = props.definitionData
    sortedDefinitionItems = buildSortedDefinitionItems(props.definitionData)
}

function containsPosition(item: SortedRangeItem, position: monaco.Position): boolean {
    const startCmp = comparePosition(position.lineNumber, position.column, item.startLine, item.startChar)
    if (startCmp < 0) {
        return false
    }
    const endCmp = comparePosition(position.lineNumber, position.column, item.endLine, item.endChar)
    return endCmp <= 0
}

function findHoverByBinarySearch(position: monaco.Position): SortedHoverItem | undefined {
    if (sortedHoverItems.length === 0) {
        return undefined
    }

    let left = 0
    let right = sortedHoverItems.length - 1
    let candidate = -1

    while (left <= right) {
        const mid = left + Math.floor((right - left) / 2)
        const item = sortedHoverItems[mid]
        if (!item) {
            break
        }
        const cmp = comparePosition(item.startLine, item.startChar, position.lineNumber, position.column)

        if (cmp <= 0) {
            candidate = mid
            left = mid + 1
        } else {
            right = mid - 1
        }
    }

    if (candidate === -1) {
        return undefined
    }

    const current = sortedHoverItems[candidate]
    if (current && containsPosition(current, position)) {
        return current
    }

    if (candidate > 0) {
        const previous = sortedHoverItems[candidate - 1]
        if (previous && containsPosition(previous, position)) {
            return previous
        }
    }

    return undefined
}

function findDefinitionByBinarySearch(position: monaco.Position): SortedDefinitionItem | undefined {
    if (sortedDefinitionItems.length === 0) {
        return undefined
    }

    let left = 0
    let right = sortedDefinitionItems.length - 1
    let candidate = -1

    while (left <= right) {
        const mid = left + Math.floor((right - left) / 2)
        const item = sortedDefinitionItems[mid]
        if (!item) {
            break
        }
        const cmp = comparePosition(item.startLine, item.startChar, position.lineNumber, position.column)

        if (cmp <= 0) {
            candidate = mid
            left = mid + 1
        } else {
            right = mid - 1
        }
    }

    if (candidate === -1) {
        return undefined
    }

    const current = sortedDefinitionItems[candidate]
    if (current && containsPosition(current, position)) {
        return current
    }

    if (candidate > 0) {
        const previous = sortedDefinitionItems[candidate - 1]
        if (previous && containsPosition(previous, position)) {
            return previous
        }
    }

    return undefined
}

function locationPathToUri(path: string): monaco.Uri {
    const normalized = path
        .split('/')
        .filter((segment) => segment.length > 0)
        .join('/')
    return monaco.Uri.parse(`file:///${normalized}`)
}

function updateHoverProvider() {
    ensureSortedHoverItems()
    hoverDispose?.dispose()
    hoverDispose = monaco.languages.registerHoverProvider(props.language, {
        provideHover(_, position) {
            const hover = findHoverByBinarySearch(position)
            if (!hover) {
                return null
            }

            const hoverRange = new monaco.Range(
                hover.startLine,
                hover.startChar,
                hover.endLine,
                hover.endChar,
            )

            if (!isPositionInRange(position, hoverRange)) {
                return null
            }

            return {
                range: hoverRange,
                contents: [{ value: hover.hover }],
            }
        },
    })
}

function updateDefinitionProvider() {
    ensureSortedDefinitionItems()
    definitionDispose?.dispose()
    definitionDispose = monaco.languages.registerDefinitionProvider(props.language, {
        provideDefinition(_, position) {
            const definition = findDefinitionByBinarySearch(position)
            if (!definition || definition.locations.length === 0) {
                return null
            }

            return definition.locations.map((location) => ({
                uri: locationPathToUri(location.fileName),
                range: new monaco.Range(
                    location.startLine,
                    location.startChar,
                    location.endLine,
                    location.endChar,
                ),
            }))
        },
    })
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
        sortedHoverSource = undefined
        updateHoverProvider()
    },
)

watch(
    () => props.definitionData,
    () => {
        sortedDefinitionSource = undefined
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