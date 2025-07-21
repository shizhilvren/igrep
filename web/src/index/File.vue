<script setup lang="ts">
import { ref, defineComponent, onMounted } from "vue";
// 定义文件路径常量

onMounted(() => {
  console.log("onMounted");
});

defineProps<{}>();

const msg = ref("some file");
</script>

<template>
  <a href="#">Link</a>
  this is a file {{ msg }}
  <p v-for="n in 200" :class="n" :id="'' + n">
    {{ n }} {{ n % 2 === 0 ? "even" : "odd" }}
    Line {{ n }}
  </p>
</template>

<style scoped></style>
