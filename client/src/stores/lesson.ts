import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { useAuthStore } from './auth'
import { useProfileStore } from './profile'
import { useVocabStore } from './vocab'

const LAST_LESSON_KEY = 'teacher_last_lesson_id'

export interface Message {
  id?: string
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
  const loopMode = ref(false)
  const lessonHistory = ref<Map<string, LessonSummary[]>>(new Map())
  const loadingHistory = ref(false)

  // TTS state
  const speakingMessageIndex = ref<number | null>(null)

  // STT state
  const isRecording = ref(false)
  const interimTranscript = ref('')

  const profileStore = useProfileStore()

  // Map human-readable language names to BCP-47 tags for Web Speech APIs
  const langMap: Record<string, string> = {
    english: 'en', spanish: 'es', french: 'fr', german: 'de', italian: 'it',
    portuguese: 'pt', dutch: 'nl', russian: 'ru', chinese: 'zh', japanese: 'ja',
    korean: 'ko', arabic: 'ar', hindi: 'hi', czech: 'cs', polish: 'pl',
    swedish: 'sv', norwegian: 'no', danish: 'da', finnish: 'fi', turkish: 'tr',
    greek: 'el', hebrew: 'he', thai: 'th', vietnamese: 'vi', romanian: 'ro',
    hungarian: 'hu', ukrainian: 'uk', indonesian: 'id', malay: 'ms',
  }

  function getLanguageTag(): string {
    const lang = profileStore.current?.language ?? 'en'
    // If it's already a short tag like "es", use it directly
    if (lang.length <= 5 && lang.includes('-') || lang.length <= 3) return lang
    // Otherwise map from name
    const tag = langMap[lang.toLowerCase()]
    if (!tag) {
      console.warn(`[speech] Unknown language "${lang}", falling back to "en". Add it to langMap in lesson store.`)
      return 'en'
    }
    return tag
  }

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
    const vocabStore = useVocabStore()
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

    vocabStore.clearLessonVocab()

    // Auto-send greeting to start the lesson
    loading.value = true
    try {
      const chatResp = await fetch('/api/lesson/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...auth.authHeaders() },
        body: JSON.stringify({
          profile_id: profileId,
          lesson_id: lesson.id,
          messages: [{ role: 'user', content: '[lesson:greeting] New lesson started. Greet the student and ask what they want to work on today. Suggest a few options based on their weak points and vocabulary.' }],
          loop_mode: false,
        }),
      })

      if (chatResp.ok) {
        const data = await chatResp.json()
        const conv = conversations.value.get(profileId)
        if (conv) {
          // Reload lesson detail to get persisted messages (only assistant reply is persisted)
          const detailResp = await fetch(`/api/lessons/${lesson.id}/detail`, {
            headers: auth.authHeaders(),
          })
          if (detailResp.ok) {
            const detail = await detailResp.json()
            conv.messages = (detail.messages ?? []).map((m: any) => ({
              id: m.id,
              role: m.role,
              content: m.content,
            }))
          } else {
            conv.messages = [{ role: 'assistant', content: data.reply }]
          }
        }
      }
    } catch (e) {
      const conv = conversations.value.get(profileId)
      if (conv) {
        conv.messages.push({ role: 'assistant', content: 'Error starting lesson.' })
      }
    } finally {
      loading.value = false
    }
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

    // Map messages from API format (with id, role, content, created_at) to store format
    const messages: Message[] = (lesson.messages ?? []).map((m: any) => ({
      id: m.id,
      role: m.role,
      content: m.content,
    }))

    conversations.value.set(profileId, {
      profileId,
      lessonId: lesson.id,
      messages,
    })

    // Load vocabulary associated with this lesson
    const vocabStore = useVocabStore()
    await vocabStore.loadLessonVocab(lessonId)
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

  async function deleteMessage(lessonId: string, messageId: string, profileId: string) {
    const auth = useAuthStore()
    const resp = await fetch(`/api/lessons/${lessonId}/messages/${messageId}`, {
      method: 'DELETE',
      headers: auth.authHeaders(),
    })

    if (resp.ok) {
      const conv = conversations.value.get(profileId)
      if (conv) {
        conv.messages = conv.messages.filter((m) => m.id !== messageId)
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
          loop_mode: loopMode.value,
        }),
      })

      const data = await resp.json()
      conv.messages.push({ role: 'assistant', content: data.reply })

      // Reload lesson to get message IDs from the database
      if (conv.lessonId) {
        const detailResp = await fetch(`/api/lessons/${conv.lessonId}/detail`, {
          headers: auth.authHeaders(),
        })
        if (detailResp.ok) {
          const lesson = await detailResp.json()
          conv.messages = (lesson.messages ?? []).map((m: any) => ({
            id: m.id,
            role: m.role,
            content: m.content,
          }))
        }

        // Reload lesson vocabulary (may have been added by Claude tools)
        const vocabStore = useVocabStore()
        await vocabStore.loadLessonVocab(conv.lessonId)
      }
    } catch (e) {
      conv.messages.push({
        role: 'assistant',
        content: 'Error communicating with the server.',
      })
    } finally {
      loading.value = false
    }
  }

  // --- TTS ---
  function stripMarkdown(text: string): string {
    return text
      .replace(/```[\s\S]*?```/g, '')       // code blocks
      .replace(/`([^`]+)`/g, '$1')          // inline code
      .replace(/!\[.*?\]\(.*?\)/g, '')       // images
      .replace(/\[([^\]]+)\]\(.*?\)/g, '$1') // links → text
      .replace(/#{1,6}\s+/g, '')            // headings
      .replace(/(\*{1,3}|_{1,3})(.*?)\1/g, '$2') // bold/italic
      .replace(/^\s*[-*+]\s+/gm, '')        // list markers
      .replace(/^\s*\d+\.\s+/gm, '')        // ordered lists
      .replace(/^\s*>\s+/gm, '')            // blockquotes
      .replace(/---+/g, '')                 // horizontal rules
      .replace(/\n{2,}/g, '\n')
      .trim()
  }

  let triedFallback = false

  function doSpeak(index: number, text: string) {
    const utterance = new SpeechSynthesisUtterance(text)
    const langTag = getLanguageTag()
    utterance.lang = langTag

    const voices = window.speechSynthesis.getVoices()
    if (voices.length === 0) {
      console.error('[TTS] No voices available. On Linux, install speech synthesis: sudo pacman -S espeak-ng speech-dispatcher')
      speakingMessageIndex.value = null
      return
    }

    // Pick a voice matching the target language — prefer local/offline voices
    const langPrefix = langTag.toLowerCase().slice(0, 2)
    const localMatch = voices.find(v => v.lang.toLowerCase().startsWith(langPrefix) && v.localService)
    const anyMatch = voices.find(v => v.lang.toLowerCase().startsWith(langPrefix))
    const match = localMatch || anyMatch

    if (match) {
      utterance.voice = match
      console.debug(`[TTS] Using voice "${match.name}" (${match.lang}, local=${match.localService}) for lang="${langTag}"`)
    } else {
      console.warn(`[TTS] No voice for "${langTag}". Available: ${voices.map(v => v.lang).join(', ')}`)
    }

    utterance.onend = () => {
      speakingMessageIndex.value = null
      triedFallback = false
    }
    utterance.onerror = (e) => {
      console.error(`[TTS] Speech error: ${e.error} (voice=${utterance.voice?.name}, lang=${langTag})`)
      speakingMessageIndex.value = null

      // Retry once without explicit voice — let the browser pick
      if (!triedFallback && match) {
        console.info('[TTS] Retrying without explicit voice selection...')
        triedFallback = true
        const retry = new SpeechSynthesisUtterance(text)
        retry.lang = langTag
        retry.onend = () => { speakingMessageIndex.value = null; triedFallback = false }
        retry.onerror = (e2) => {
          console.error(`[TTS] Retry also failed: ${e2.error}`)
          speakingMessageIndex.value = null
          triedFallback = false
        }
        speakingMessageIndex.value = index
        window.speechSynthesis.speak(retry)
      }
    }

    speakingMessageIndex.value = index
    window.speechSynthesis.speak(utterance)
  }

  function speakMessage(index: number) {
    const msg = currentMessages.value[index]
    if (!msg || msg.role !== 'assistant') return

    // Stop if already speaking this message
    if (speakingMessageIndex.value === index) {
      window.speechSynthesis.cancel()
      speakingMessageIndex.value = null
      return
    }

    window.speechSynthesis.cancel()

    const text = stripMarkdown(msg.content)

    // Voices may not be loaded yet — wait for them if needed
    if (window.speechSynthesis.getVoices().length === 0) {
      window.speechSynthesis.onvoiceschanged = () => {
        window.speechSynthesis.onvoiceschanged = null
        doSpeak(index, text)
      }
      // Trigger voice loading
      window.speechSynthesis.getVoices()
    } else {
      doSpeak(index, text)
    }
  }

  function stopSpeaking() {
    window.speechSynthesis.cancel()
    speakingMessageIndex.value = null
  }

  // --- STT ---
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const SpeechRecognitionImpl: any =
    (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition

  let recognition: any = null

  function startRecording(onResult: (text: string) => void) {
    if (!SpeechRecognitionImpl) {
      console.error('[STT] SpeechRecognition API not available in this browser')
      return
    }

    recognition = new SpeechRecognitionImpl()
    recognition.lang = getLanguageTag()
    recognition.interimResults = true
    recognition.continuous = true

    recognition.onresult = (event: any) => {
      let interim = ''
      let finalText = ''
      for (let i = event.resultIndex; i < event.results.length; i++) {
        const transcript = event.results[i][0].transcript
        if (event.results[i].isFinal) {
          finalText += transcript
        } else {
          interim += transcript
        }
      }
      // Clear interim when final result arrives to avoid duplication
      interimTranscript.value = finalText ? '' : interim
      if (finalText) onResult(finalText)
    }

    recognition.onerror = (e: any) => {
      console.error('[STT] Recognition error:', e.error)
      stopRecording()
    }
    recognition.onend = () => { isRecording.value = false }

    isRecording.value = true
    interimTranscript.value = ''
    recognition.start()
  }

  function stopRecording() {
    recognition?.stop()
    recognition = null
    isRecording.value = false
    interimTranscript.value = ''
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

  // Persist last lesson per profile
  watch(currentLessonId, (lessonId) => {
    const profileId = profileStore.current?.id
    if (!profileId) return
    const saved = JSON.parse(localStorage.getItem(LAST_LESSON_KEY) ?? '{}')
    if (lessonId) {
      saved[profileId] = lessonId
    } else {
      delete saved[profileId]
    }
    localStorage.setItem(LAST_LESSON_KEY, JSON.stringify(saved))
  })

  async function restoreLastLesson() {
    const profileId = profileStore.current?.id
    if (!profileId) return
    if (conversations.value.get(profileId)?.lessonId) return
    const saved = JSON.parse(localStorage.getItem(LAST_LESSON_KEY) ?? '{}')
    const lessonId = saved[profileId]
    if (lessonId) {
      await loadLesson(lessonId)
    }
  }

  return {
    conversations,
    currentLessonId,
    currentMessages,
    loading,
    loopMode,
    loadingHistory,
    lessonHistory,
    speakingMessageIndex,
    isRecording,
    interimTranscript,
    ensureConversation,
    startNewLesson,
    loadLesson,
    loadLessonHistory,
    deleteLesson,
    deleteMessage,
    sendMessage,
    clearMessages,
    speakMessage,
    stopSpeaking,
    startRecording,
    stopRecording,
    restoreLastLesson,
  }
})
