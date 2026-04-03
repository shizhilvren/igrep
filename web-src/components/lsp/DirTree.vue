<template>
    <main>
        <nav class="path-nav" aria-label="breadcrumb" style="--bs-breadcrumb-divider: ''">
            <el-table v-if="fileRows.length != 0" :data="fileRows" stripe style="width: 100%; margin-bottom: 16px">
                <el-table-column prop="name" label="File">
                    <template #default="{ row }">
                        <a :href="row.url">{{ row.name }}</a>
                    </template>
                </el-table-column>
            </el-table>

            <el-table v-if="dirRows.length != 0" :data="dirRows" stripe style="width: 100%">
                <el-table-column prop="name" label="Directory">
                    <template #default="{ row }">
                        <a :href="row.url">{{ row.name }}</a>
                    </template>
                </el-table-column>
            </el-table>
        </nav>
    </main>
</template>



<script setup lang="ts">
import { computed } from 'vue'

type TableRow = {
    name: string
    url: string
}

const props = defineProps<{
    files: string[]
    dirs: string[]
    base_path: string[]
}>()

function getUrl(base_path: string[], entry: string): string {
    if (base_path.length == 0) {
        return `#/files/${[entry].join('/')}`
    } else {
        return `#/files/${[...base_path, entry].join('/')}`
    }
}

const fileRows = computed<TableRow[]>(() => props.files.map((name) => ({
    name,
    url: getUrl(props.base_path, name),
})))

const dirRows = computed<TableRow[]>(() => props.dirs.map((name) => ({
    name,
    url: getUrl(props.base_path, name),
})))
</script>

<style scoped></style>
