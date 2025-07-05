<script setup lang="ts">
import { ref, defineComponent, onMounted } from 'vue'
import { FileLinesResult, LineResult } from '../typescript/search.ts';


onMounted(async () => {

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
    </div>
</template>


<style scoped></style>
