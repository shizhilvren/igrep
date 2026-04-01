<template>
    <main>
        this is files view
        <div>{{ Array.isArray(filePath) ? filePath.join(', ') : filePath }}</div>
    </main>
</template>




<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { fetchFileData, OneLineRange } from '@/utils/utils';
import { SearchOneFileLinesContentResult, SearchEngine, VecU8 } from 'igrep';
import SearchBox from '@/components/search/SearchBox.vue';
import FileResult from '@/components/search/FileResult.vue';
import { LineContent } from '@/components/search/FileResult.vue';
import { CancellablePromise } from 'real-cancellable-promise';

const props = defineProps<{
    filePath: string | string[]
}>()

onMounted(() => {
    console.log('SearchView mounted');
});


</script>

<script lang="ts">



</script>