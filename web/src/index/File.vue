<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { ref, defineComponent, onMounted, watch } from "vue";
const route = ref(useRoute());
const router = ref(useRouter());

onMounted(() => {
  console.log("onMounted");
});


watch(
  () => route.value.params.pathMatch,
  (newPath) => {
    console.log("path changed", newPath);
    let path = "../../index/" + newPath.join("/") + ".json";
    console.log("path", path);
    const json_path: URL = new URL(path, import.meta.url);
    fetchData(json_path).then((data) => {
      console.log("data", data);
      file.value = data;
    });
  },
  { immediate: true }
);
defineProps<{}>();

const msg = ref("some file");
const file = ref({ path: "not load", content: { content: [{ tokens: [{ token: "empty" }] }] } });

// In a Vue component method or lifecycle hook (e.g., created or mounted)
async function fetchData(path: URL) {
  console.log("fetching data from", path);
  try {
    const response = await fetch(path); // Replace with your JSON URL
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const jsonData = await response.json();
    return jsonData;
  } catch (error) {
    console.error("Error fetching JSON:", error);
  }
}
</script>

<template>
  <p>{{ $route.params.pathMatch }}</p>
  file name {{ file.path }}
  <el-row v-for="(line, id) in file.content.content" style="">
    <el-col :span="1" :id="'' + (id + 1)">
      <div style="float: right; padding-right: 10px;">
        {{ id + 1 }}
      </div>
    </el-col>
    <el-col :span="23">
      <pre><span v-for="token in line.tokens" style=""><span>{{ token.token }}</span></span></pre>
    </el-col>
  </el-row>
  <!-- <p>{{ file }}</p> -->
</template>

<style scoped></style>
