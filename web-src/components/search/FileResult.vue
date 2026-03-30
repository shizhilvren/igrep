<template>
    <div v-if="0 != prop.lines.length">
        <div>
            <button @click="show = !show" class="btn btn-sm" style="width: 6ch;">
                {{ show ? 'Hide' : 'Show' }}
            </button>
            {{ prop.filePath }}
        </div>
        <div :id="'file' + prop.id" v-if="show">
            <CodeViewLines v-bind="{
                language: 'cpp',
                value: prop.lines.map(line => line.content),
                lineNumbers: prop.lines.map(line => line.lineNum),
                highlightColNumberRanges: prop.lines.map(line => line.highlightColNumberRanges),
            }" />
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
import CodeViewOneLine from './CodeViewLines.vue'
import CodeViewLines from './CodeViewLines.vue';
import { ref } from 'vue';

const prop = defineProps<{
    filePath: string
    id: number
    lines: LineContent[]
}>();

const show = ref(true)

</script>

<style scoped>
/* .collapse {
    &:not(.show) {
        height: 100px;
        display: none;
    }
} */
</style>