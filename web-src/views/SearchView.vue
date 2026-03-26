<script setup lang="ts">
import { onMounted, ref } from 'vue';

import { fetchFileData } from '@/utils/utils';
import { greet, SearchEngine } from 'igrep';
import SearchBox from '@/components/search/SearchBox.vue';

onMounted(() => {
    console.log('SearchView mounted');
    // greet('World');
});

function handleSearch(searchTerm: string) {
    console.log('Search term:', searchTerm);
    // Here you can add logic to perform the search using the searchTerm
    let search_one_engine = search_engine.value?.search(searchTerm);
    search_one_engine?.ngrams().vec().map((ngram) => {
        console.log('Ngram:', ngram);
        let ngram_path = ngram.path_str("ngram-index")
        console.log('Ngram path:', ngram_path);
        fetchFileData(ngram_path).then((data) => {
            console.log('Ngram file data loaded:', data);
            // You can process the data as needed here
        }).catch((error) => {
            console.error('Error loading ngram file data:', error);
            alert('Failed to load ngram data. Please try again later.');
        });
    });
}

const init_finished = ref(false);
const search_engine = ref<SearchEngine | null>(null);

onMounted(() => {
    fetchFileData("ngram-index/global.data").then((data) => {
        console.log('File data loaded:', data);
        // You can process the data as needed here
        search_engine.value = new SearchEngine(data);
    }).catch((error) => {
        console.error('Error loading file data:', error);
        alert('Failed to load search data. Please try again later.');
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
