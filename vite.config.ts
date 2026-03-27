import { fileURLToPath, URL } from 'node:url'
import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

const projectRoot = fileURLToPath(new URL('./', import.meta.url));
const ngramRoot = resolve(projectRoot, 'ngram-index');

function addRawNgramMiddleware(middlewares: { use: (path: string, handler: (req: any, res: any, next: () => void) => void) => void }) {
  middlewares.use('/ngram-index', async (req, res, next) => {
    const reqUrl = req.url || '/';
    const pathname = decodeURIComponent(reqUrl.split('?')[0] || '/');
    const resolvedPath = resolve(ngramRoot, `.${pathname}`);

    if (!resolvedPath.startsWith(ngramRoot)) {
      res.statusCode = 403;
      res.end('Forbidden');
      return;
    }

    try {
      const data = await readFile(resolvedPath);
      res.statusCode = 200;
      res.end(data);
      return;
    } catch (error: any) {
      if (error?.code === 'ENOENT' || error?.code === 'EISDIR') {
        next();
        return;
      }
      res.statusCode = 500;
      res.end('Failed to read ngram file');
    }
  });
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    {
      name: 'raw-ngram-index-files',
      configureServer(server) {
        addRawNgramMiddleware(server.middlewares);
      },
      configurePreviewServer(server) {
        addRawNgramMiddleware(server.middlewares);
      },
    },
    wasm(),
    topLevelAwait(),
    vue(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./web-src', import.meta.url)),
      '@root': fileURLToPath(new URL('./', import.meta.url)),
    },
  },
  base: process.env.VITE_BASE_URL || '/',
})
