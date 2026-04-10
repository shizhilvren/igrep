<template>
    <div ref="el" class="monaco-container"></div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import loader from '@monaco-editor/loader'
import * as monaco from 'monaco-editor'
import { OneLineRange } from '@/utils/utils'

const el = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

let sizeDispose: monaco.IDisposable | null = null
let decorations: monaco.editor.IEditorDecorationsCollection | null = null
let hoverDispose: monaco.IDisposable | null = null

function syncEditorHeight() {
    if (!editor || !el.value) return
    const contentHeight = editor.getContentHeight()
    el.value.style.height = `${contentHeight}px`
    editor.layout({
        width: el.value.clientWidth,
        height: contentHeight,
    })
}

function highlightText(ranges: OneLineRange[]) {
    if (!editor || !ranges) return
    const model = editor.getModel()
    if (!model) return

    const next = ranges.map((range) => ({
        range: new monaco.Range(1, range.startCollNumber, 1, range.endCollNumber),
        options: { inlineClassName: 'my-string-highlight' as const },
    }))

    if (!decorations) {
        decorations = editor.createDecorationsCollection(next)
        return
    }
    decorations.set(next)
    // editor.setSelection(new monaco.Range(2, 1, 2, 10))
    editor.setSelection("a".toString() as any)
    editor.focus()
}



onMounted(async () => {
    loader.config({ monaco })
    await loader.init()

    if (!el.value) {
        return
    }

    editor = monaco.editor.create(el.value, {
        value: [
            'fn main() {',
            '    let name = "igrep";a',
            '    println!("hello {}", name);aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaasssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
            '}',
        ].join('\n'),
        language: 'rust',
        theme: 'vs',
        readOnly: true,
        //     domReadOnly: true,
        // cursorStyle: 'line-thin',
        // cursorBlinking: 'solid',
        // hideCursorInOverviewRuler: true,
        // renderLineHighlight: 'none',
        // overviewRulerLanes: 0,
        hover: { enabled: true },
        automaticLayout: true,
        wordWrap: 'off',
        minimap: { enabled: false },
        lineNumbers: (num) => String(num + 30),
        scrollbar: {
            handleMouseWheel: false,
            vertical: 'hidden',
            horizontal: 'auto',
        },
    })

    hoverDispose = monaco.languages.registerHoverProvider('rust', {
        provideHover(model, position) {
            console.debug('Hover at', position);
            const word = model.getWordAtPosition(position)
            if (!word) return null

            if (word.word === 'name') {
                return {
                    range: new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn),
                    contents: [
                        { value: '### variable `bCOUJqDnJG`  \n\n---\nType: `const char *`  \nValue = `&\"bCOUJqDnJG\"[0]`  \nPassed as name  \n\n---\n```cpp\n// In main\nconst char *bCOUJqDnJG = \"bCOUJqDnJG\"\n```' },
                    ],
                }
            }

            if (word.word === 'println') {
                return {
                    range: new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn),
                    contents: [
                        { value: '**宏:** `println!`' },
                        { value: '向标准输出打印格式化字符串。' },
                    ],
                }
            }

            return null
        },
    })

    syncEditorHeight()
    sizeDispose = editor.onDidContentSizeChange(() => {
        syncEditorHeight()
    })
    highlightText([new OneLineRange(2, 10,), new OneLineRange(1, 2)])
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
    /* height: 420px; */
    /* overflow: hidden; */
    /* border: 1px solid #ddd; */
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