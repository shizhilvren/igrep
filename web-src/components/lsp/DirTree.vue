<template>
    <main>
        <nav class="path-nav" aria-label="breadcrumb" style="--bs-breadcrumb-divider: ''">
            files
            <div v-for="(f, i) in files" :key="i">
                <a :href="getUrl(props.base_path, f)">{{ f }}</a>
            </div>
            dirs
            <div v-for="(d, i) in dirs" :key="i">
                <a :href="getUrl(props.base_path, d)">{{ d }}</a>
            </div>
        </nav>
    </main>
</template>



<script setup lang="ts">

const props = defineProps<{
    files: string[]
    dirs: string[]
    base_path: string[]
}>()
</script>
<script lang="ts">
function getUrl(base_path: string[], file: string): string {
    if (base_path.length == 0) {
        return `#/files/${[file].join('/')}`

    } else {

        return `#/files/${[...base_path, file].join('/')}`
    }
}
</script>

<style scoped></style>
