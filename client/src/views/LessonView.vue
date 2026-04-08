<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { marked } from 'marked'
import { useLessonStore } from '../stores/lesson'
import { useProfileStore } from '../stores/profile'

const lessonStore = useLessonStore()
const profileStore = useProfileStore()

const input = ref('')
const chatContainer = ref<HTMLElement | null>(null)

function renderMarkdown(text: string): string {
  return marked.parse(text, { async: false }) as string
}

async function send() {
  const text = input.value.trim()
  if (!text || lessonStore.loading) return
  input.value = ''
  await lessonStore.sendMessage(text)
  await nextTick()
  chatContainer.value?.scrollTo({ top: chatContainer.value.scrollHeight, behavior: 'smooth' })
}

async function newLesson() {
  await lessonStore.startNewLesson()
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
</script>

<template>
  <div class="flex flex-col h-[calc(100vh-3.5rem)]">
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
      </div>

      <!-- Chat messages -->
      <div ref="chatContainer" class="flex-1 overflow-y-auto p-4 space-y-4">
        <div v-if="lessonStore.currentMessages.length === 0" class="text-center text-gray-400 mt-20">
          <p class="text-lg">Start a conversation in {{ profileStore.current.language }} ({{ profileStore.current.level }})</p>
        </div>

        <div
          v-for="(msg, i) in lessonStore.currentMessages"
          :key="i"
          :class="[
            'max-w-[80%] rounded-lg px-4 py-3',
            msg.role === 'user'
              ? 'ml-auto bg-blue-600 text-white'
              : 'mr-auto bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700'
          ]"
        >
          <div
            v-if="msg.role === 'assistant'"
            class="prose dark:prose-invert prose-sm max-w-none"
            v-html="renderMarkdown(msg.content)"
          />
          <p v-else class="whitespace-pre-wrap">{{ msg.content }}</p>
        </div>

        <div v-if="lessonStore.loading" class="mr-auto text-gray-400 text-sm animate-pulse">
          Thinking...
        </div>
      </div>

      <!-- Input -->
      <div class="border-t border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-800">
        <form @submit.prevent="send" class="flex gap-2">
          <input
            v-model="input"
            type="text"
            placeholder="Type your message..."
            class="flex-1 rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            :disabled="lessonStore.loading"
          />
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
</template>
