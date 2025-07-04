import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  // 如果您想修改项目根目录，取消下面这行的注释并设置您想要的路径
  // root: '.',
  
  // 如果您想将应用部署到子路径，取消下面这行的注释并设置您的子路径
  base: '/~lisimon/igrep/vue-project/dist/',
  
  plugins: [
    vue(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      // 添加对pkg文件夹的别名，方便引入
      'igrep_pkg': fileURLToPath(new URL('./pkg/igrep.js', import.meta.url))
    },
  },
  // 确保WebAssembly文件能被正确处理
  optimizeDeps: {
    exclude: ['igrep_pkg']  // 排除WebAssembly包以防止预构建优化问题
  },
  // 如果您想自定义构建输出目录
  // build: {
  //   outDir: './dist-custom'
  // }
})
