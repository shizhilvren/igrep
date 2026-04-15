<script setup lang="ts">
import MonacoView from '../components/MonacoView.vue'
import VMView from './VMView.vue';
</script>

<template>
  <main>
    <MonacoView />
    <VMView style="height: 500px; width: 500px;" />
  </main>
</template>
