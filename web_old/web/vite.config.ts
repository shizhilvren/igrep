import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
 base: "/~lisimon/igrep/web/dist/",
    plugins: [
    vue(),
    vueDevTools(),
  ],
  assetsInclude: ["**.*wasm"],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
})
