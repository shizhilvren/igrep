<script setup lang="ts">
import HelloWorld from "./components/HelloWorld.vue";
import TheWelcome from "./components/TheWelcome.vue";
import Search from "./views/Search.vue";
import File from "./index/File.vue";
</script>

<template>
  <p><strong>Current route path:</strong> {{ $route.fullPath }}</p>
  <nav>
    <RouterLink to="/search">Go to search</RouterLink>
    <RouterLink to="/index">Go to Index</RouterLink>
  </nav>
  <main>
  </main>
  <main>
    <div class="common-layout">
      <el-row class="row-bg" justify="center">
        <el-col :span="16">
          <div class="grid-content ep-bg-purple">
            <RouterView />
          </div>
        </el-col>
      </el-row>
    </div>
  </main>
</template>

<style scoped>
/* header {
  line-height: 1.5;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
} */

/* @media (min-width: 1024px) {
  header {
    display: flex;
    place-items: center;
    padding-right: calc(var(--section-gap) / 4);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }
} */
</style>
