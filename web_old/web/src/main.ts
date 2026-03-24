import './assets/main.css'

import { createApp } from 'vue'
import App from './App.vue'
import ElementPlus from 'element-plus'
import 'highlight.js/styles/stackoverflow-light.css'
import 'highlight.js/lib/common';
// import hljsVuePlugin from "@highlightjs/vue-plugin";
import hljsVuePlugin from "./typescript/vue-highlight";

import 'element-plus/dist/index.css'
import router from './typescript/router'

let app = createApp(App)
app.use(ElementPlus)
app.use(hljsVuePlugin)
app.use(router)

app.mount('#app')

