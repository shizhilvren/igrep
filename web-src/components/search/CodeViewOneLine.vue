<template>
    <div ref="el" class="monaco-container"></div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import loader from '@monaco-editor/loader'
import * as monaco from 'monaco-editor'
import { OneLineRange } from '@/utils/utils'


const props = defineProps<{
    value: string
    language: string
    lineNumbers: number
    highlightColNumberRanges: OneLineRange[]
}>();

const el = ref<HTMLElement | null>(null)
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let editor: monaco.editor.IStandaloneCodeEditor | null = null
let sizeDispose: monaco.IDisposable | null = null

onMounted(async () => {
    loader.config({ monaco })
    await loader.init()

    if (!el.value) {
        return
    }

    editor = monaco.editor.create(el.value, {
        value: [
            props.value,
        ].join('\n'),
        language: props.language,
        theme: 'vs',
        readOnly: true,
        automaticLayout: true,
        wordWrap: 'off',
        minimap: { enabled: false },
        lineNumbers: (num) => String(num + props.lineNumbers - 1),
        scrollbar: {
            handleMouseWheel: false,
            vertical: 'hidden',
            horizontal: 'auto',
        },
    })
    syncEditorHeight()
    sizeDispose = editor.onDidContentSizeChange((e) => {
        syncEditorHeight()
    })
    console.debug('Highlighting text with ranges:', props.highlightColNumberRanges);
    highlightText(props.highlightColNumberRanges)
})

onBeforeUnmount(() => {
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
    editor.layout({
        width: el.value.clientWidth,
        height: contentHeight,
    })
}

function highlightText(ranges: OneLineRange[]) {
    if (!editor || !ranges) return
    const model = editor.getModel()
    if (!model) return
    console.debug('Highlighting text with ranges:', ranges);

    const next = ranges.map((range) => ({
        range: new monaco.Range(1, range.startCollNumber, 1, range.endCollNumber),
        options: { inlineClassName: 'my-string-highlight' as const },
    }))

    if (!decorations) {
        decorations = editor.createDecorationsCollection(next)
        return
    }
    decorations.set(next)
}

</script>

<style scoped>
.monaco-container {
    /* height: 420px; */
    /* overflow: hidden; */
    /* border: 1px solid #ddd; */
}

:deep(.my-string-highlight) {
    background: rgba(255, 220, 120, 0.45);
    border-radius: 2px;
}
</style>