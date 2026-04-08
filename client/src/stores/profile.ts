import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from './auth'

export interface Profile {
  id: string
  user_id: string
  language: string
  level: string
  style: string
  explanation_language: string
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
      current.value = profiles.value[0]
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
    data: { level?: string; style?: string; explanation_language?: string }
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

  return { profiles, current, loadProfiles, createProfile, updateProfile }
})
