import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { useAuthStore } from './auth'

const LAST_PROFILE_KEY = 'teacher_last_profile_id'

export interface Profile {
  id: string
  user_id: string
  language: string
  level: string
  style: string
  explanation_language: string
  personal_note: string
}

export const useProfileStore = defineStore('profile', () => {
  const profiles = ref<Profile[]>([])
  const current = ref<Profile | null>(null)

  async function loadProfiles() {
    const auth = useAuthStore()
    const resp = await fetch('/api/profiles', {
      headers: { ...auth.authHeaders() },
    })
    profiles.value = await resp.json()
    if (profiles.value.length > 0 && !current.value) {
      const lastId = localStorage.getItem(LAST_PROFILE_KEY)
      const saved = lastId ? profiles.value.find((p) => p.id === lastId) : null
      current.value = saved ?? profiles.value[0]
    }
  }

  async function createProfile(data: {
    language: string
    level?: string
    style?: string
    explanation_language?: string
  }) {
    const auth = useAuthStore()
    const resp = await fetch('/api/profiles', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
      body: JSON.stringify(data),
    })
    const profile: Profile = await resp.json()
    profiles.value.push(profile)
    current.value = profile
    return profile
  }

  async function updateProfile(
    id: string,
    data: { level?: string; style?: string; explanation_language?: string; personal_note?: string }
  ) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/profiles/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
      body: JSON.stringify(data),
    })
    const updated: Profile = await resp.json()
    const idx = profiles.value.findIndex((p) => p.id === id)
    if (idx >= 0) profiles.value[idx] = updated
    if (current.value?.id === id) current.value = updated
    return updated
  }

  async function deleteProfile(id: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/profiles/${id}`, {
      method: 'DELETE',
      headers: auth.authHeaders(),
    })

    if (resp.ok) {
      profiles.value = profiles.value.filter((p) => p.id !== id)
      if (current.value?.id === id) {
        current.value = profiles.value.length > 0 ? profiles.value[0] : null
      }
    }
  }

  watch(current, (profile) => {
    if (profile) {
      localStorage.setItem(LAST_PROFILE_KEY, profile.id)
    } else {
      localStorage.removeItem(LAST_PROFILE_KEY)
    }
  })

  return { profiles, current, loadProfiles, createProfile, updateProfile, deleteProfile }
})
