<template>
    <div v-if="0 != prop.lines.length">
        <div>{{ prop.filePath }}</div>
        <CodeViewLines v-bind="{
            language: 'cpp',
            value: prop.lines.map(line => line.content),
            lineNumbers: prop.lines.map(line => line.lineNum),
            highlightColNumberRanges: prop.lines.map(line => line.highlightColNumberRanges),
        }" />
    </div>
</template>


<script lang="ts">
export class LineContent {
    readonly lineNum: number
    readonly content: string
    readonly highlightColNumberRanges: OneLineRange[]
    constructor(lineNum: number, content: string, highlightColNumberRanges: OneLineRange[]) {
        this.lineNum = lineNum
        this.content = content
        this.highlightColNumberRanges = highlightColNumberRanges
    }
};
</script>

<script setup lang="ts">
import { OneLineRange } from '@/utils/utils';
import CodeViewOneLine from './CodeViewLines.vue'
import CodeViewLines from './CodeViewLines.vue';

const prop = defineProps<{
    filePath: string
    lines: LineContent[]
}>();

</script>