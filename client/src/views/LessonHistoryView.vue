<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useLessonStore } from '../stores/lesson'
import { useProfileStore } from '../stores/profile'

const lessonStore = useLessonStore()
const profileStore = useProfileStore()
const router = useRouter()

async function loadForCurrentProfile() {
  if (profileStore.current) {
    await lessonStore.loadLessonHistory(profileStore.current.id)
  }
}

onMounted(loadForCurrentProfile)

watch(() => profileStore.current?.id, loadForCurrentProfile)

function openLesson(lessonId: string) {
  if (!profileStore.current) return
  lessonStore.loadLesson(lessonId)
  router.push('/lesson')
}

async function deleteLesson(lessonId: string) {
  if (!profileStore.current) return
  await lessonStore.deleteLesson(lessonId, profileStore.current.id)
}

function formatDate(dateStr: string): string {
  const d = new Date(dateStr)
  return d.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="p-4 max-w-2xl mx-auto">
    <h2 class="text-xl font-semibold mb-6">Lesson History</h2>

    <div v-if="!profileStore.current" class="text-center text-gray-500 mt-8">
      <p>No profile selected. Go to <router-link to="/settings" class="text-blue-600 underline">Settings</router-link>.</p>
    </div>

    <template v-else>
      <div v-if="lessonStore.loadingHistory" class="text-center text-gray-400 mt-8">
        Loading...
      </div>

      <div
        v-else-if="!lessonStore.lessonHistory.get(profileStore.current.id)?.length"
        class="text-center text-gray-400 mt-8"
      >
        No lessons yet for {{ profileStore.current.language }}.
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="lesson in lessonStore.lessonHistory.get(profileStore.current.id)"
          :key="lesson.id"
          class="flex items-center justify-between rounded-lg border border-gray-200 dark:border-gray-700 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800 group"
        >
          <button
            @click="openLesson(lesson.id)"
            class="flex-1 text-left min-w-0"
          >
            <div class="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
              {{ lesson.title }}
            </div>
            <div class="text-xs text-gray-400">
              {{ formatDate(lesson.updated_at) }} · {{ lesson.message_count }} messages
            </div>
          </button>
          <button
            @click.stop="deleteLesson(lesson.id)"
            class="ml-2 p-1 text-gray-300 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
            title="Delete lesson"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>
    </template>
  </div>
</template>
