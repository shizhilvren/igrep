<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue'
import TheWelcome from './components/TheWelcome.vue'
import Search from './views/Search.vue';

</script>

<template>
  <main>
    <div class="common-layout">
      <el-container>
        <el-header>Header</el-header>
        <el-row class="row-bg" justify="center">
          <el-col :span="18">
            <div class="grid-content ep-bg-purple">
              <Search />
            </div>
          </el-col>
        </el-row>
        <el-footer>Footer</el-footer>
      </el-container>
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
