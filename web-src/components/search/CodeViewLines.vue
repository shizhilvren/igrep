<template>
    <div ref="el" class="monaco-container"></div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import loader from '@monaco-editor/loader'
import * as monaco from 'monaco-editor'
import { OneLineRange } from '@/utils/utils'


const props = defineProps<{
    language: string
    value: string[]
    lineNumbers: number[]
    highlightColNumberRanges: OneLineRange[][]
}>();

const el = ref<HTMLElement | null>(null)
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let editor: monaco.editor.IStandaloneCodeEditor | null = null
let sizeDispose: monaco.IDisposable | null = null
let removeFirstClickGuard: (() => void) | null = null

onMounted(async () => {
    loader.config({ monaco })
    await loader.init()

    if (!el.value) {
        return
    }

    const host = el.value
    const guardFirstClickScrollJump = () => {
        const scrollX = window.scrollX
        const scrollY = window.scrollY
        requestAnimationFrame(() => {
            window.scrollTo(scrollX, scrollY)
        })
        host.removeEventListener('mousedown', guardFirstClickScrollJump)
        removeFirstClickGuard = null
    }
    host.addEventListener('mousedown', guardFirstClickScrollJump)
    removeFirstClickGuard = () => {
        host.removeEventListener('mousedown', guardFirstClickScrollJump)
    }

    editor = monaco.editor.create(el.value, {
        value: [
            ...props.value,
        ].join('\n'),
        language: props.language,
        theme: 'vs',
        readOnly: true,
        folding: false,
        showFoldingControls: 'never',
        // Monaco readOnly is enough; domReadOnly can cause first-focus scroll jumps in long content.
        domReadOnly: false,
        // cursorStyle: 'line-thin',
        // cursorBlinking: 'solid',
        // hideCursorInOverviewRuler: true,
        // renderLineHighlight: 'none',
        // overviewRulerLanes: 0,
        automaticLayout: true,
        wordWrap: 'off',
        minimap: { enabled: false },
        lineNumbers: (num) => String(props.lineNumbers[num - 1]!),
        scrollbar: {
            handleMouseWheel: false,
            vertical: 'hidden',
            horizontal: 'auto',
        },
    })
    syncEditorHeight()
    // editor.focus()
    sizeDispose = editor.onDidContentSizeChange((e) => {
        syncEditorHeight()
    })
    console.debug('Highlighting text with ranges:', props.highlightColNumberRanges);
    highlightText(props.highlightColNumberRanges)
})

onBeforeUnmount(() => {
    removeFirstClickGuard?.()
    removeFirstClickGuard = null
    sizeDispose?.dispose()
    sizeDispose = null
    decorations?.clear()
    decorations = null
    editor?.dispose()
    editor = null
})


function syncEditorHeight() {
    if (!editor || !el.value) return
    const contentHeight = editor.getContentHeight()
    console.debug('Syncing editor height to content height:', contentHeight);
    editor.layout({
        width: el.value.clientWidth,
        height: contentHeight,
    })
}

function highlightText(ranges: OneLineRange[][]) {
    if (!editor || !ranges) return
    const model = editor.getModel()
    if (!model) return
    console.debug('Highlighting text with ranges:', ranges);

    const next = ranges.map((range, lineIndex) => (
        range.map((r) => (

            {
                range: new monaco.Range(lineIndex + 1, r.startCollNumber, lineIndex + 1, r.endCollNumber),
                options: { inlineClassName: 'my-string-highlight' as const },
            }
        )
        ))).flat()

    if (!decorations) {
        decorations = editor.createDecorationsCollection(next)
        return
    }
    decorations.set(next)
}

</script>

<style scoped>
.monaco-container {
    width: 100%;
    /* height: 420px; */
    /* overflow: hidden; */
    /* border: 1px solid #ddd; */
}

:deep(.my-string-highlight) {
    background: rgba(255, 220, 120, 0.45);
    border-radius: 2px;
}

:deep(.monaco-editor .view-overlays .current-line) {
    border: 0 !important;
}
</style>