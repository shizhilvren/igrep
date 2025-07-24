<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { ref, defineComponent, nextTick, onMounted, watch } from "vue";
import { fetchJson } from "@/typescript/utils"; // Assuming you have a utility function to fetch data
const projectRoot = "../.."
const route = ref(useRoute());
const router = ref(useRouter());
const symbolJsonPrefix = `${projectRoot}/index/symbol/`;
onMounted(() => {
  console.log("onMounted");
});


watch(
  () => route.value.params.pathMatch,
  (newPath) => {
    console.log("path changed", newPath);
    if (typeof newPath === "string") {
      let path = symbolJsonPrefix + newPath + ".json";
      console.log("path", path);
      const json_path: URL = new URL(path, import.meta.url);
      fetchJson(json_path).then((data) => {
        console.log("data", data);
        symbol.value = data;
        nextTick().then(() => {
          const el = document.getElementById("1");
          router.value.push(route.value.fullPath);
        });
      });
    } else {
      console.error("newPath is not a string", newPath);
    }
  },
  { immediate: true }
);
defineProps<{}>();

const msg = ref("some file");
const symbol = ref({
  name: "empty_name",
  definition: [{
    "file": "file_name",
    "line": 1522
  }],
  declaration: [
    {
      "file": "file_name",
      "line": 1522
    }
  ],
  call: [{
    "file": "file_name",
    "line": 1522
  }]
});

// In a Vue component method or lifecycle hook (e.g., created or mounted)

</script>

<template>
  <p>{{ $route.params.pathMatch }}</p>
  <h2>Symbol: {{ symbol.name }}</h2>
  <h2 v-show="symbol.definition.length != 0">Definition: ({{ symbol.definition.length }})</h2>
  <el-row v-for="def in symbol.definition">
    <el-col :span="24">
      <pre>{{ def.file }}:{{ def.line }}</pre>
    </el-col>
  </el-row>
  <h2 v-show="symbol.declaration.length != 0">Declaration: ({{ symbol.declaration.length }})</h2>
  <el-row v-for="decl in symbol.declaration">
    <el-col :span="24">
      <pre>{{ decl.file }}:{{ decl.line }}</pre>
    </el-col>
  </el-row>
  <h2 v-show="symbol.call.length != 0">Call: ({{ symbol.call.length }})</h2>
  <el-row v-for="call in symbol.call">
    <el-col :span="24">
      <pre>{{ call.file }}:{{ call.line }}</pre>
    </el-col>
  </el-row>
</template>

<style scoped>
.fn {
  color: #00677c;
}
</style>
