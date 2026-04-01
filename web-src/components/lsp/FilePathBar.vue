<template>
    <main>
        <nav class="path-nav" aria-label="breadcrumb" style="--bs-breadcrumb-divider: ''">
            <div v-for="(p, i) in filePath" class="breadcrumb-item" :class="{ active: i === filePath.length - 1 }"
                aria-current="page" :key="i">
                <a :href="getUrl(filePath, i)">{{ p }}</a>
                <div class="divider">
                    {{ i === filePath.length - 1 ? '' : '/' }}
                </div>
            </div>
        </nav>
    </main>
</template>



<script setup lang="ts">

const props = defineProps<{
    filePath: string[]
}>()
</script>
<script lang="ts">
function getUrl(filePath: string[], index: number): string {
    return `#/files/${filePath.slice(0, index + 1).join('/')}`
}
</script>

<style scoped>
.path-nav {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    /* gap: 6px; */
}

.breadcrumb-item {
    display: flex;
    align-items: center;
    white-space: nowrap;
}

.breadcrumb-item a {
    text-decoration: none;
}

.breadcrumb-item a:hover {
    text-decoration: underline;
}

.divider {
    margin: 0 4px;
}
</style>
