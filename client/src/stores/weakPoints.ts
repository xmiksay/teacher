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

  return { items, load }
})
