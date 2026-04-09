import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from './auth'

export interface WeakPoint {
  id: string
  profile_id: string
  type: string
  detail: string
  active: boolean
}

export const useWeakPointsStore = defineStore('weakPoints', () => {
  const items = ref<WeakPoint[]>([])

  async function load(profileId: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/weak-points/${profileId}`, {
      headers: { ...auth.authHeaders() },
    })
    items.value = await resp.json()
  }

  async function add(profileId: string, type: string, detail: string) {
    const auth = useAuthStore()
    const resp = await fetch('/api/weak-points', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
      body: JSON.stringify({ profile_id: profileId, type, detail }),
    })
    const created: WeakPoint = await resp.json()
    items.value.push(created)
  }

  async function remove(id: string) {
    const auth = useAuthStore()
    await fetch(`/api/weak-points/${id}/delete`, {
      method: 'DELETE',
      headers: { ...auth.authHeaders() },
    })
    items.value = items.value.filter((wp) => wp.id !== id)
  }

  return { items, load, add, remove }
})
