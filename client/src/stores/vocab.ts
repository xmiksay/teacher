import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from './auth'

export interface VocabWord {
  id: string
  profile_id: string
  word: string
  translation: string
  added_by: string
  context: string | null
  last_practiced: string
  error_count: number
}

export const useVocabStore = defineStore('vocab', () => {
  const words = ref<VocabWord[]>([])
  const lessonWords = ref<VocabWord[]>([])
  const flashcardIndex = ref(0)

  async function loadVocab(profileId: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/vocab/${profileId}?limit=200`, {
      headers: { ...auth.authHeaders() },
    })
    words.value = await resp.json()
    flashcardIndex.value = 0
  }

  async function addWord(data: {
    profile_id: string
    word: string
    translation: string
    context?: string
  }) {
    const auth = useAuthStore()
    const resp = await fetch('/api/vocab', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
      body: JSON.stringify(data),
    })
    const word: VocabWord = await resp.json()
    words.value.push(word)
  }

  async function deleteWord(id: string) {
    const auth = useAuthStore()
    await fetch(`/api/vocab/${id}/delete`, { method: 'DELETE', headers: { ...auth.authHeaders() } })
    words.value = words.value.filter((w) => w.id !== id)
  }

  async function deleteAll(profileId: string) {
    const auth = useAuthStore()
    await fetch(`/api/vocab/${profileId}/delete-all`, {
      method: 'DELETE',
      headers: { ...auth.authHeaders() },
    })
    words.value = []
    flashcardIndex.value = 0
  }

  async function loadLessonVocab(lessonId: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/vocab/lesson/${lessonId}`, {
      headers: { ...auth.authHeaders() },
    })
    if (resp.ok) {
      lessonWords.value = await resp.json()
    }
  }

  function clearLessonVocab() {
    lessonWords.value = []
  }

  function nextFlashcard() {
    if (words.value.length > 0) {
      flashcardIndex.value = (flashcardIndex.value + 1) % words.value.length
    }
  }

  return { words, lessonWords, flashcardIndex, loadVocab, loadLessonVocab, clearLessonVocab, addWord, deleteWord, deleteAll, nextFlashcard }
})
