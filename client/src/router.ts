import { createRouter, createWebHistory } from 'vue-router'
import LoginView from './views/LoginView.vue'
import LessonView from './views/LessonView.vue'
import VocabView from './views/VocabView.vue'
import WeakPointsView from './views/WeakPointsView.vue'
import LessonHistoryView from './views/LessonHistoryView.vue'
import SettingsView from './views/SettingsView.vue'
import { useAuthStore } from './stores/auth'

const routes = [
  { path: '/login', component: LoginView, meta: { public: true } },
  { path: '/', redirect: '/lesson' },
  { path: '/lesson', component: LessonView },
  { path: '/lesson-history', component: LessonHistoryView },
  { path: '/vocab', component: VocabView },
  { path: '/weak-points', component: WeakPointsView },
  { path: '/settings', component: SettingsView },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to) => {
  const auth = useAuthStore()
  if (!to.meta.public && !auth.isLoggedIn) {
    return '/login'
  }
  if (to.path === '/login' && auth.isLoggedIn) {
    return '/lesson'
  }
})
