<script setup lang="ts">
import { onMounted, watch, computed, ref } from 'vue'
import { useWeakPointsStore } from '../stores/weakPoints'
import { useProfileStore } from '../stores/profile'

const weakPointsStore = useWeakPointsStore()
const profileStore = useProfileStore()

const activeItems = computed(() => weakPointsStore.items.filter((wp) => wp.active))
const resolvedItems = computed(() => weakPointsStore.items.filter((wp) => !wp.active))

const newType = ref('grammar')
const newDetail = ref('')

async function loadForCurrentProfile() {
  if (profileStore.current) {
    await weakPointsStore.load(profileStore.current.id)
  }
}

async function addWeakPoint() {
  if (!profileStore.current || !newDetail.value.trim()) return
  await weakPointsStore.add(profileStore.current.id, newType.value, newDetail.value.trim())
  newDetail.value = ''
}

async function removeWeakPoint(id: string) {
  await weakPointsStore.remove(id)
}

onMounted(loadForCurrentProfile)

watch(() => profileStore.current?.id, loadForCurrentProfile)
</script>

<template>
  <div class="p-4 max-w-2xl mx-auto">
    <h2 class="text-xl font-semibold mb-6">Weak Points</h2>

    <div v-if="!profileStore.current" class="text-center text-gray-500 mt-8">
      <p>No profile selected. Go to <router-link to="/settings" class="text-blue-600 underline">Settings</router-link>.</p>
    </div>

    <template v-else>
      <!-- Add weak point form -->
      <form @submit.prevent="addWeakPoint" class="mb-6 flex gap-2 items-end">
        <select v-model="newType" class="rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm">
          <option value="grammar">Grammar</option>
          <option value="vocabulary">Vocabulary</option>
          <option value="phrase">Phrase</option>
        </select>
        <input
          v-model="newDetail"
          type="text"
          placeholder="Describe the weak point..."
          class="flex-1 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm"
        />
        <button
          type="submit"
          :disabled="!newDetail.trim()"
          class="rounded-lg bg-blue-600 text-white px-4 py-2 text-sm hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Add
        </button>
      </form>

      <!-- Active weak points -->
      <div class="mb-8">
        <h3 class="text-sm font-medium text-gray-500 mb-3">Active</h3>
        <div v-if="activeItems.length === 0" class="text-gray-400 text-sm">
          No active weak points. Great job!
        </div>
        <div class="space-y-2">
          <div
            v-for="wp in activeItems"
            :key="wp.id"
            class="rounded-lg border border-gray-200 dark:border-gray-700 px-4 py-3 text-sm flex items-center justify-between"
          >
            <div>
              <span class="inline-block rounded bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs px-2 py-0.5 mr-2">
                {{ wp.type }}
              </span>
              {{ wp.detail }}
            </div>
            <button
              @click="removeWeakPoint(wp.id)"
              class="text-gray-400 hover:text-red-500 ml-2 text-xs"
              title="Remove"
            >
              &times;
            </button>
          </div>
        </div>
      </div>

      <!-- Resolved weak points -->
      <div v-if="resolvedItems.length > 0">
        <h3 class="text-sm font-medium text-gray-500 mb-3">Resolved</h3>
        <div class="space-y-2">
          <div
            v-for="wp in resolvedItems"
            :key="wp.id"
            class="rounded-lg border border-gray-100 dark:border-gray-800 px-4 py-3 text-sm text-gray-400 flex items-center justify-between"
          >
            <div>
              <span class="inline-block rounded bg-gray-100 dark:bg-gray-800 text-gray-400 text-xs px-2 py-0.5 mr-2">
                {{ wp.type }}
              </span>
              <span class="line-through">{{ wp.detail }}</span>
            </div>
            <button
              @click="removeWeakPoint(wp.id)"
              class="text-gray-400 hover:text-red-500 ml-2 text-xs"
              title="Remove"
            >
              &times;
            </button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
