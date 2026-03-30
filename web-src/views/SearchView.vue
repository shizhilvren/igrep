<template>
    <main>
        this is search
        <SearchBox hit_msg="Enter search term" button_msg="Search" :disable="!init_finished" @search="handleSearch" />
        <div>Files: {{ number_results.files_count }}, Lines: {{ number_results.lines_count }}</div>
        <FileResult v-for="(item, index) in search_item" :key="index" v-bind="{
            id: index,
            filePath: item.full_file_name()
            , lines: item.lines().map(
                line => {
                    return new LineContent(
                        line.line_num(),
                        line.content(),
                        line.match_range().map(r => new OneLineRange(r.start + 1, r.end + 1))
                    )
                })
        }" />
    </main>
</template>




<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { fetchFileData, OneLineRange } from '@/utils/utils';
import { SearchOneFileLinesContentResult, SearchEngine, VecU8 } from 'igrep';
import SearchBox from '@/components/search/SearchBox.vue';
import FileResult from '@/components/search/FileResult.vue';
import { LineContent } from '@/components/search/FileResult.vue';

onMounted(() => {
    console.log('SearchView mounted');
    // greet('World');
});

const init_finished = ref(false);
const search_engine = ref<SearchEngine | null>(null);
const search_item = ref<SearchOneFileLinesContentResult[]>([])
const number_results = ref(new ResultCount(0, 0))

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

async function handleSearch(searchTerm: string) {
    console.log('Search term:', searchTerm);
    number_results.value = new ResultCount(0, 0);
    search_item.value = [];
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
    for (let [idx, iter] of files_data_promise.entries()) {
        iter.then((data) => {
            console.log('File data loaded:', data);
            let file_data = new VecU8(data)
            let date_index = files_index[idx]!;
            let file_match = search_one_engine?.file_lines_match(date_index, file_data, files_lines_index)!;
            if (file_match.is_empty()) {
                console.log('No matches found in file:', date_index);
                return;
            }
            let lines = file_match.lines();
            number_results.value.lines_count += lines.length;
            number_results.value.files_count += 1;
            lines.forEach((line) => {
                console.log('Matched line:', line);
            });
            search_item.value.push(file_match);
        }).catch((error) => {
            console.error('Error loading file data:', error);
        });
    }
}
</script>

<script lang="ts">

class ResultCount {
    public lines_count: number
    public files_count: number
    constructor(lines_count: number, files_count: number) {
        this.lines_count = lines_count
        this.files_count = files_count
    }
}
</script>