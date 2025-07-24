<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { ref, defineComponent, nextTick, onMounted, watch } from "vue";

import { fetchJson } from "@/typescript/utils"; // Assuming you have a utility function to fetch data
const projectRoot = "../.."
const route = ref(useRoute());
const router = ref(useRouter());
const fileJsonPrefix = `${projectRoot}/index/file/`;
onMounted(() => {
  console.log("onMounted");
});


watch(
  () => route.value.params.pathMatch,
  (newPath) => {
    console.log("path changed", newPath);
    let path = fileJsonPrefix + newPath.join("/") + ".json";
    console.log("path", path);
    const json_path: URL = new URL(path, import.meta.url);
    fetchJson(json_path).then((data) => {
      console.log("data", data);
      file.value = data;
      nextTick().then(() => {
        const el = document.getElementById("1");
        router.value.push(route.value.fullPath);
      });
    });
  },
  { immediate: true }
);
defineProps<{}>();

function makeLink(id: string, classes: string[]) {
  let link = "";
  if (classes.includes("fn")) {
    let symbol = btoa(id);
    link = `/symbol/${symbol}`;
  }
  return link;
}

const msg = ref("some file");
const file = ref({ path: "not load", content: { content: [{ tokens: [{ token: "empty", classes: [], id: "" }] }] } });

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
      <span v-for="token in line.tokens">
        <router-link v-if="makeLink(token.id, token.classes) !== ''" :to="makeLink(token.id, token.classes)"
          style="text-decoration: none; color: inherit;">
          <span style="white-space: pre;" :class="token.classes" :id="token.id"
            @click="makeLink(token.id, token.classes)">
            {{ token.token }}
          </span>
        </router-link>
        <span v-else>
          <span style="white-space: pre;" :class="token.classes" :id="token.id"
            @click="makeLink(token.id, token.classes)">
            {{ token.token }}
          </span>
        </span>
      </span>
    </el-col>
  </el-row>
</template>

<style scoped>
.fn {
  color: #00677c;
}
</style>
