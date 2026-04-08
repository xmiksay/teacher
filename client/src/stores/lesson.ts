import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useAuthStore } from './auth'
import { useProfileStore } from './profile'

export interface Message {
  role: 'user' | 'assistant'
  content: string
}

export interface LessonSummary {
  id: string
  profile_id: string
  title: string
  created_at: string
  updated_at: string
  message_count: number
}

interface Conversation {
  profileId: string
  lessonId: string | null
  messages: Message[]
}

export const useLessonStore = defineStore('lesson', () => {
  const conversations = ref<Map<string, Conversation>>(new Map())
  const loading = ref(false)
  const lessonHistory = ref<Map<string, LessonSummary[]>>(new Map())
  const loadingHistory = ref(false)

  const profileStore = useProfileStore()

  const currentMessages = computed(() => {
    const id = profileStore.current?.id
    if (!id) return []
    return conversations.value.get(id)?.messages ?? []
  })

  const currentLessonId = computed(() => {
    const id = profileStore.current?.id
    if (!id) return null
    return conversations.value.get(id)?.lessonId ?? null
  })

  function ensureConversation(profileId: string) {
    if (!conversations.value.has(profileId)) {
      conversations.value.set(profileId, { profileId, lessonId: null, messages: [] })
    }
  }

  async function startNewLesson() {
    const profileId = profileStore.current?.id
    if (!profileId) return

    const auth = useAuthStore()
    const resp = await fetch('/api/lessons', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
      body: JSON.stringify({ profile_id: profileId }),
    })

    if (!resp.ok) return

    const lesson = await resp.json()
    conversations.value.set(profileId, {
      profileId,
      lessonId: lesson.id,
      messages: [],
    })
  }

  async function loadLesson(lessonId: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/lessons/${lessonId}/detail`, {
      headers: auth.authHeaders(),
    })

    if (!resp.ok) return

    const lesson = await resp.json()
    const profileId = lesson.profile_id

    // Switch global profile to match the lesson
    const profile = profileStore.profiles.find((p) => p.id === profileId)
    if (profile) {
      profileStore.current = profile
    }

    conversations.value.set(profileId, {
      profileId,
      lessonId: lesson.id,
      messages: lesson.messages ?? [],
    })
  }

  async function loadLessonHistory(profileId: string) {
    const auth = useAuthStore()
    loadingHistory.value = true
    try {
      const resp = await fetch(`/api/lessons/${profileId}`, {
        headers: auth.authHeaders(),
      })
      if (resp.ok) {
        const data = await resp.json()
        lessonHistory.value.set(profileId, data)
      }
    } finally {
      loadingHistory.value = false
    }
  }

  async function deleteLesson(lessonId: string, profileId: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/lessons/${lessonId}/delete`, {
      method: 'DELETE',
      headers: auth.authHeaders(),
    })

    if (resp.ok) {
      const history = lessonHistory.value.get(profileId)
      if (history) {
        lessonHistory.value.set(
          profileId,
          history.filter((l) => l.id !== lessonId),
        )
      }
      const conv = conversations.value.get(profileId)
      if (conv?.lessonId === lessonId) {
        conv.lessonId = null
        conv.messages = []
      }
    }
  }

  async function sendMessage(text: string) {
    const profileId = profileStore.current?.id
    if (!profileId) return

    ensureConversation(profileId)
    const conv = conversations.value.get(profileId)!

    // Auto-create a lesson if none is active
    if (!conv.lessonId) {
      await startNewLesson()
    }

    conv.messages.push({ role: 'user', content: text })
    loading.value = true

    try {
      const auth = useAuthStore()
      const resp = await fetch('/api/lesson/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
        body: JSON.stringify({
          profile_id: profileId,
          lesson_id: conv.lessonId,
          messages: conv.messages,
        }),
      })

      const data = await resp.json()
      conv.messages.push({ role: 'assistant', content: data.reply })
    } catch (e) {
      conv.messages.push({
        role: 'assistant',
        content: 'Error communicating with the server.',
      })
    } finally {
      loading.value = false
    }
  }

  function clearMessages(profileId?: string) {
    const id = profileId ?? profileStore.current?.id
    if (id) {
      const conv = conversations.value.get(id)
      if (conv) {
        conv.messages = []
        conv.lessonId = null
      }
    }
  }

  return {
    conversations,
    currentLessonId,
    currentMessages,
    loading,
    loadingHistory,
    lessonHistory,
    ensureConversation,
    startNewLesson,
    loadLesson,
    loadLessonHistory,
    deleteLesson,
    sendMessage,
    clearMessages,
  }
})
