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


function class_name(name: string) {
    return "." + name;
}
</script>

<template>
    <div v-bind:class="file_line_result.uuid">
        <el-affix :offset="50" :target="class_name(file_line_result.uuid)">
            <div class="file-header">
                {{ file_line_result.name }}
            </div>
        </el-affix>
        <p v-for="item in file_line_result.lines" class="">
            <ResultLine :line_number="item.line" :line_string="item.string" :ranges="item.match" language="cpp" />
        </p>
    </div>
</template>


<style>
.hljs {
    padding: 0 !important;
    /* background-color: #fff !important; */

}

</style>
<style scoped>
.file-header {
    padding-left: 20px;
    padding-top: 2px;
    padding-bottom: 2px;
    background-color: #eef2f5;
}
</style>
