<template>
    <div v-if="0 != prop.lines.length">
        <div>{{ prop.filePath }}</div>
        <div v-for="line in prop.lines" :key="line.lineNum">
            <CodeViewOneLine :value="line.content" language="cpp" :lineNumbers="line.lineNum"
                :highlightColNumberRanges="line.highlightColNumberRanges" />
        </div>
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
import CodeViewOneLine from './CodeViewOneLine.vue'

const prop = defineProps<{
    filePath: string
    lines: LineContent[]
}>();

</script>