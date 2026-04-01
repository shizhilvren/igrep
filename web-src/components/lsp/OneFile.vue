<template>
    <div ref="el" class="monaco-container"></div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
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
}>()



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
})

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



:deep(.monaco-editor .cursor) {
    display: none !important;
}

/* :deep(.monaco-editor .view-overlays .current-line) {
    border: 0 !important;
} */
</style>