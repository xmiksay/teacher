import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface AuthState {
  userId: string
  token: string
  expiresAt: string
}

const STORAGE_KEY = 'teacher_auth'

function loadFromStorage(): AuthState | null {
  const raw = localStorage.getItem(STORAGE_KEY)
  if (!raw) return null
  try {
    const state: AuthState = JSON.parse(raw)
    if (new Date(state.expiresAt) < new Date()) {
      localStorage.removeItem(STORAGE_KEY)
      return null
    }
    return state
  } catch {
    localStorage.removeItem(STORAGE_KEY)
    return null
  }
}

export const useAuthStore = defineStore('auth', () => {
  const saved = loadFromStorage()
  const userId = ref(saved?.userId ?? '')
  const token = ref(saved?.token ?? '')
  const expiresAt = ref(saved?.expiresAt ?? '')

  const isLoggedIn = computed(() => !!token.value)

  function persist() {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ userId: userId.value, token: token.value, expiresAt: expiresAt.value })
    )
  }

  async function register(username: string, password: string) {
    const resp = await fetch('/api/auth/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    })
    if (!resp.ok) {
      const text = await resp.text()
      throw new Error(text || 'Registration failed')
    }
    const data = await resp.json()
    userId.value = data.user_id
    token.value = data.token
    expiresAt.value = data.expires_at
    persist()
  }

  async function login(username: string, password: string) {
    const resp = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    })
    if (!resp.ok) {
      const text = await resp.text()
      throw new Error(text || 'Login failed')
    }
    const data = await resp.json()
    userId.value = data.user_id
    token.value = data.token
    expiresAt.value = data.expires_at
    persist()
  }

  function logout() {
    userId.value = ''
    token.value = ''
    expiresAt.value = ''
    localStorage.removeItem(STORAGE_KEY)
  }

  function authHeaders(): Record<string, string> {
    return token.value ? { Authorization: `Bearer ${token.value}` } : {}
  }

  return { userId, token, expiresAt, isLoggedIn, register, login, logout, authHeaders }
})
