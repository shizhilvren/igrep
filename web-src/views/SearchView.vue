<script setup lang="ts">
import { onMounted } from 'vue';
import { greet } from 'igrep';
onMounted(() => {
  console.log('SearchView mounted');
  greet('World');
});



</script>

<template>
  <main>
    this is search
  </main>
</template>
