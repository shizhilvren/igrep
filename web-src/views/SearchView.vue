<script setup lang="ts">
import { onMounted, ref } from 'vue';

import { fetchFileData } from '@/utils/utils';
import { greet, SearchEngine, VecU8 } from 'igrep';
import SearchBox from '@/components/search/SearchBox.vue';

onMounted(() => {
    console.log('SearchView mounted');
    // greet('World');
});

async function handleSearch(searchTerm: string) {
    console.log('Search term:', searchTerm);
    // Here you can add logic to perform the search using the searchTerm
    let search_one_engine = search_engine.value?.search(searchTerm);
    let ngrams = search_one_engine?.ngrams();
    let a = ngrams?.vec().map((ngram) => {
        console.log('Ngram:', ngram);
        let ngram_path = ngram.path_str("ngram-index")
        console.log('Ngram path:', ngram_path);
        return fetchFileData(ngram_path);
    });
    if (!a || !ngrams) {
        throw new Error("No search engine available or no ngrams found");
    }
    let results = (await Promise.all(a)).map((data) => {
        return new VecU8(data);
    });
    let files_lines_index = search_one_engine?.files_lines(ngrams, results);
    let files_index = files_lines_index?.files();
    let files_data_promise = files_index?.map((file_index) => {
        console.log('File index:', file_index);
        return fetchFileData(file_index.path_str("ngram-index") + "/file");
    });
    if (!files_data_promise || !files_index || !files_lines_index) {
        throw new Error("No files data found");
    }
    let files_data = (await Promise.all(files_data_promise)).map((data) => {
        return new VecU8(data);
    });
    files_data.forEach((data, idx) => {
        console.log('File data:', data);
        let date_index = files_index[idx];
        if (!date_index) {
            console.error('No file index for data at index:', idx);
            return;
        }
        let ans = search_one_engine?.file_lines_match(date_index, data, files_lines_index);
        console.log('File lines match result:', ans);
    });

}

const init_finished = ref(false);
const search_engine = ref<SearchEngine | null>(null);

onMounted(() => {
    fetchFileData("ngram-index/global.data").then((data) => {
        console.log('File data loaded:', data);
        // You can process the data as needed here
        search_engine.value = new SearchEngine(new VecU8(data));
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
