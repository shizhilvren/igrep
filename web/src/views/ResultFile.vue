<script setup lang="ts">
import { ref, defineComponent, useTemplateRef, onMounted, onUpdated } from 'vue'
import { FileLinesResult, LineResult } from '../typescript/search.ts';
import ResultLine, { } from './ResultLine.vue';
import hljs from 'highlight.js/lib/core'
// import { escapeHtml } from '.,/typescript//utils'
import type { BeforeHighlightContext, HighlightResult } from 'highlight.js';
onMounted(async () => {
    // console.log('highlightjs component updated', pRef.value[0].$refs.code)
    // hljs.highlightElement(pRef.value[0] as HTMLElement)

})


onUpdated(() => {
    // console.log('highlightjs component updated', code.value)
})

defineProps<{
    file_line_result: FileLinesResult
}>()


</script>

<template>
    <div>
        <p>
            {{ file_line_result.name }}
        </p>
        <p v-for="item in file_line_result.lines" class="scrollbar-demo-item">
            <ResultLine :line_number="item.line" :line_string="item.string" :ranges="item.match"  language="cpp"/>
        </p>
    </div>
</template>


<style>
.hljs {
    padding: 0 !important;
}
</style>
