import { createWebHashHistory, createWebHistory, createRouter } from 'vue-router'
import Search from '../views/Search.vue'
import Index from '../index/File.vue'

const routes = [
  { path: '/search', component: Search },
  { path: '/index/:pathMatch(.*)*', component: Index },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (to.hash) {
      return {
        el: to.hash,
      }
    }
  }
})

export default router