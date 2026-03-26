<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { fetchFileData } from '@/utils/utils';
import { greet } from 'igrep';
import SearchBox from '@/components/search/SearchBox.vue';

onMounted(() => {
  console.log('SearchView mounted');
  // greet('World');
});

function handleSearch(searchTerm: string) {
  console.log('Search term:', searchTerm);
  // Here you can add logic to perform the search using the searchTerm
}

const init_finished = ref(false);

onMounted(() => {
  fetchFileData("ngram-index/global.data").then((data) => {
    console.log('File data loaded:', data);
    // You can process the data as needed here
  }).catch((error) => {
    console.error('Error loading file data:', error);
  }).finally(() => {
    init_finished.value = true;
  });
});



</script>

<template>
  <main>
    this is search
    <SearchBox hit_msg="Enter search term" button_msg="Search" :disable="!init_finished" @search="handleSearch" />
  </main>
</template>
