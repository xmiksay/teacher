<script setup lang="ts">
import { ref, nextTick, watch, computed, onMounted } from 'vue'
import { marked } from 'marked'
import { useLessonStore } from '../stores/lesson'
import { useProfileStore } from '../stores/profile'
import { useVocabStore } from '../stores/vocab'

const lessonStore = useLessonStore()
const profileStore = useProfileStore()
const vocabStore = useVocabStore()

const input = ref('')
const chatContainer = ref<HTMLElement | null>(null)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const vocabOpen = ref(false)

const hasTTS = typeof window !== 'undefined' && 'speechSynthesis' in window
const hasSTT = typeof window !== 'undefined' && ('SpeechRecognition' in window || 'webkitSpeechRecognition' in window)

const displayInput = computed(() => {
  if (lessonStore.isRecording && lessonStore.interimTranscript) {
    return input.value + (input.value ? ' ' : '') + lessonStore.interimTranscript
  }
  return input.value
})

function renderMarkdown(text: string): string {
  return marked.parse(text, { async: false }) as string
}

function autoResize() {
  const el = textareaRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 150) + 'px'
}

async function send(e?: Event) {
  e?.preventDefault()
  const text = input.value.trim()
  if (!text || lessonStore.loading) return
  input.value = ''
  await nextTick()
  autoResize()
  await lessonStore.sendMessage(text)
  await nextTick()
  chatContainer.value?.scrollTo({ top: chatContainer.value.scrollHeight, behavior: 'smooth' })
}

async function newLesson() {
  await lessonStore.startNewLesson()
}

function deleteMessage(messageId: string) {
  const lessonId = lessonStore.currentLessonId
  const profileId = profileStore.current?.id
  if (!lessonId || !profileId) return
  lessonStore.deleteMessage(lessonId, messageId, profileId)
}

function toggleRecording() {
  if (lessonStore.isRecording) {
    lessonStore.stopRecording()
  } else {
    lessonStore.startRecording((text) => {
      input.value += (input.value ? ' ' : '') + text
    })
  }
}

// Ensure conversation exists when profile changes
watch(
  () => profileStore.current?.id,
  (id) => {
    if (id) lessonStore.ensureConversation(id)
  },
  { immediate: true }
)

watch(
  () => lessonStore.currentMessages.length,
  async () => {
    await nextTick()
    chatContainer.value?.scrollTo({ top: chatContainer.value.scrollHeight, behavior: 'smooth' })
  }
)

onMounted(async () => {
  await nextTick()
  chatContainer.value?.scrollTo({ top: chatContainer.value.scrollHeight })
})
</script>

<template>
  <div class="flex h-[calc(100vh-3.5rem)]">
    <!-- Main chat column -->
    <div class="flex flex-col flex-1 min-w-0">
    <!-- No profile selected -->
    <div v-if="!profileStore.current" class="flex-1 flex items-center justify-center text-gray-400">
      <p>No profile selected. Go to <router-link to="/settings" class="text-blue-600 underline">Settings</router-link>.</p>
    </div>

    <template v-else>
      <!-- Lesson toolbar -->
      <div class="flex items-center gap-2 px-4 py-2 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
        <button
          @click="newLesson"
          class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-green-700"
        >
          New lesson
        </button>
        <label class="flex items-center gap-1.5 text-xs text-gray-600 dark:text-gray-400 cursor-pointer select-none">
          <input
            type="checkbox"
            v-model="lessonStore.loopMode"
            class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500"
          />
          Loop mode
        </label>
        <div class="flex-1" />
        <button
          v-if="vocabStore.lessonWords.length > 0"
          @click="vocabOpen = !vocabOpen"
          class="relative rounded-lg px-2 py-1.5 text-xs font-medium transition-colors"
          :class="vocabOpen
            ? 'bg-blue-600 text-white hover:bg-blue-700'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'"
          title="Toggle vocabulary"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
          </svg>
          <span class="absolute -top-1.5 -right-1.5 bg-blue-600 text-white text-[10px] font-bold rounded-full w-4 h-4 flex items-center justify-center">
            {{ vocabStore.lessonWords.length }}
          </span>
        </button>
      </div>

      <!-- Chat messages -->
      <div ref="chatContainer" class="flex-1 overflow-y-auto p-4 space-y-4">
        <div v-if="lessonStore.currentMessages.length === 0" class="text-center text-gray-400 mt-20">
          <p class="text-lg">Start a conversation in {{ profileStore.current.language }} ({{ profileStore.current.level }})</p>
        </div>

        <div
          v-for="(msg, i) in lessonStore.currentMessages"
          :key="msg.id ?? i"
          :class="[
            'max-w-[80%] rounded-lg px-4 py-3 group relative',
            msg.role === 'user'
              ? 'ml-auto bg-blue-600 text-white'
              : 'mr-auto bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700'
          ]"
        >
          <!-- Delete message button -->
          <button
            v-if="msg.id && lessonStore.currentLessonId"
            @click="deleteMessage(msg.id)"
            class="absolute -top-2 opacity-0 group-hover:opacity-100 transition-opacity bg-white dark:bg-gray-700 rounded-full p-0.5 shadow text-gray-400 hover:text-red-500"
            :class="msg.role === 'user' ? '-left-2' : '-right-2'"
            title="Delete message"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          <div v-if="msg.role === 'assistant'">
            <div
              class="prose dark:prose-invert prose-sm max-w-none"
              v-html="renderMarkdown(msg.content)"
            />
            <button
              v-if="hasTTS"
              @click="lessonStore.speakMessage(i)"
              class="mt-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 text-xs flex items-center gap-1"
              :title="lessonStore.speakingMessageIndex === i ? 'Stop' : 'Listen'"
            >
              <svg v-if="lessonStore.speakingMessageIndex !== i" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072M17.95 6.05a8 8 0 010 11.9M11 5L6 9H2v6h4l5 4V5z" />
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10h6v4H9z" />
              </svg>
              <span>{{ lessonStore.speakingMessageIndex === i ? 'Stop' : 'Listen' }}</span>
            </button>
          </div>
          <p v-else class="whitespace-pre-wrap">{{ msg.content }}</p>
        </div>

        <div v-if="lessonStore.loading" class="mr-auto text-gray-400 text-sm animate-pulse">
          Thinking...
        </div>
      </div>

      <!-- Input -->
      <div class="border-t border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-800">
        <form @submit.prevent="send" class="flex gap-2">
          <textarea
            ref="textareaRef"
            v-model="input"
            :placeholder="lessonStore.isRecording ? displayInput || 'Listening...' : 'Type your message...'"
            class="flex-1 rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none overflow-hidden"
            :class="{ 'ring-2 ring-red-400': lessonStore.isRecording }"
            :disabled="lessonStore.loading"
            rows="1"
            @input="autoResize"
            @keydown.enter.exact="send"
          />
          <button
            v-if="hasSTT"
            type="button"
            @click="toggleRecording"
            class="rounded-lg px-3 py-2 text-sm font-medium transition-colors"
            :class="lessonStore.isRecording
              ? 'bg-red-500 text-white hover:bg-red-600 animate-pulse'
              : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'"
            :disabled="lessonStore.loading"
            :title="lessonStore.isRecording ? 'Stop recording' : 'Start recording'"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path v-if="!lessonStore.isRecording" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4M12 15a3 3 0 003-3V5a3 3 0 00-6 0v7a3 3 0 003 3z" />
              <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /><path v-if="lessonStore.isRecording" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10h6v4H9z" />
            </svg>
          </button>
          <button
            type="submit"
            class="rounded-lg bg-blue-600 px-6 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
            :disabled="lessonStore.loading || !input.trim()"
          >
            Send
          </button>
        </form>
      </div>
    </template>
    </div>

    <!-- Vocabulary sidebar overlay (mobile) / inline (desktop) -->
    <Transition name="vocab-slide">
      <div
        v-if="vocabOpen && vocabStore.lessonWords.length > 0"
        class="fixed inset-y-0 right-0 z-30 w-72 border-l border-gray-200 dark:border-gray-700 overflow-y-auto bg-gray-50 dark:bg-gray-900 p-3 shadow-lg md:static md:shadow-none md:z-auto md:w-64 md:shrink-0"
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
            Lesson Vocabulary
          </h3>
          <button
            @click="vocabOpen = false"
            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 md:hidden"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div
          v-for="word in vocabStore.lessonWords"
          :key="word.id"
          class="mb-2 p-2 bg-white dark:bg-gray-800 rounded-lg text-sm border border-gray-200 dark:border-gray-700"
        >
          <div class="font-medium text-gray-900 dark:text-gray-100">{{ word.word }}</div>
          <div class="text-gray-500 dark:text-gray-400 text-xs">{{ word.translation }}</div>
          <div v-if="word.context" class="text-gray-400 dark:text-gray-500 text-xs mt-1 italic">{{ word.context }}</div>
        </div>
      </div>
    </Transition>

    <!-- Backdrop for mobile overlay -->
    <div
      v-if="vocabOpen && vocabStore.lessonWords.length > 0"
      class="fixed inset-0 z-20 bg-black/30 md:hidden"
      @click="vocabOpen = false"
    />
  </div>
</template>

<style scoped>
.vocab-slide-enter-active,
.vocab-slide-leave-active {
  transition: transform 0.2s ease;
}
.vocab-slide-enter-from,
.vocab-slide-leave-to {
  transform: translateX(100%);
}
</style>
