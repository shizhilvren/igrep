import { createRouter, createWebHashHistory, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import SearchView from '@/views/SearchView.vue'
import FilesView from '@/views/FilesView.vue'
import FileView from '@/views/FileView.vue'
import VMView from '@/views/VMView.vue'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/search',
      name: 'search',
      component: SearchView,
    },
    {
      path: '/about',
      name: 'about',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AboutView.vue'),
    },
    {
      path: "/files/:filePath(.*)*",
      name: "files",
      component: FileView,
      props: route => ({ filePath: route.params.filePath }),
    },
    {
      path: '/vm',
      name: 'vm',
      component: VMView,
    }
  ],
})

export default router
