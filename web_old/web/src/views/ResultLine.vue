<script setup lang="ts">
import { ref, defineComponent, useTemplateRef, onMounted, onUpdated } from 'vue'
import { FileLinesResult, LineResult } from '../typescript/search.ts';
import hljs from 'highlight.js/lib/core'
// import { escapeHtml } from '.,/typescript//utils'
import type { BeforeHighlightContext, HighlightResult } from 'highlight.js';
import type { FileDataMatchRange } from '../../pkg/igrep';
onMounted(async () => {


})

onUpdated(() => {
    // console.log('highlightjs component updated', code.value)
})


const props = defineProps<{
    line_string: string,
    line_number: number,
    ranges: Array<FileDataMatchRange>,
    language: string,
}>()


const line = ref();
</script>

<template>
    <div>
        <el-row>
            <el-col :span="1" class="alignment-container">
                <pre><code class="hljs" tabindex="0" ><div style="float: right; padding-right: 10px;">{{ line_number }}</div></code></pre>
            </el-col>
            <el-col :span="23">
                <highlightjs :language='language' :code="line_string" :ranges="ranges" />
                <!-- <pre><code ref="code">{{ line_string }}</code></pre>  -->
            </el-col>
        </el-row>
    </div>
</template>


<style>
.hljs {
    padding: 0 !important;
}

.match-highlight {
    background-color: var(--mark-bg)
}

:root {
    --mark-bg: #f8e688;
}
</style>
