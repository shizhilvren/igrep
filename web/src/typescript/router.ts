import { createWebHashHistory, createWebHistory, createRouter } from 'vue-router'
import Search from '@/views/Search.vue'
import Index from '@/index/File.vue'
import Symbol from '@/index/Symbol.vue'

const routes = [
  { path: '/search', component: Search },
  { path: '/file/:pathMatch(.*)*', component: Index },
  { path: '/symbol/:pathMatch(.*)', component: Symbol },

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

router.beforeEach((to, from) => {
  // ...
  // 返回 false 以取消导航
  if (to.path.startsWith('/index')) {
    console.log('Navigation to ignored path, canceling navigation.')
    return false
  }
  return true
})

export default router