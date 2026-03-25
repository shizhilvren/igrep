<script setup lang="ts">
import { onMounted } from 'vue';
import { greet } from 'igrep';
import SearchBox from '@/components/search/SearchBox.vue';
onMounted(() => {
  console.log('SearchView mounted');
  greet('World');
});

function handleSearch(searchTerm: string) {
  console.log('Search term:', searchTerm);
  // Here you can add logic to perform the search using the searchTerm
}

</script>

<template>
  <main>
    this is search
    <SearchBox hit_msg="Enter search term" button_msg="Search" :disable="false" @search="handleSearch" />
  </main>
</template>
