import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ui from '@nuxt/ui/vue-plugin'
import i18n from './i18n'
import App from './App.vue'
import { createRouter, createWebHistory } from 'vue-router'

const app = createApp(App)

const router = createRouter({
  routes: [],
  history: createWebHistory()
})

app.use(router)
app.use(createPinia())
app.use(ui)
app.use(i18n)

app.mount('#app')
