<script setup lang="ts">
import { ref } from 'vue';

defineProps<{
    hit_msg: string,
    button_msg: string,
    disable: boolean
}>()
const inputValue = ref('')
</script>

<template>
    <div class="input-group mb-3">
        <input v-model="inputValue" type="text" class="form-control" :placeholder="hit_msg" aria-label="Search"
            aria-describedby="button-addon2" >
        <button @click="$emit('search', inputValue)" class="btn btn-primary" type="button" id="button-addon2" :disabled="disable">{{ button_msg }} </button>
    </div>
</template>

<style scoped></style>
