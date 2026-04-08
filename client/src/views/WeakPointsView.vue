<script setup lang="ts">
import { onMounted, watch, computed } from 'vue'
import { useWeakPointsStore } from '../stores/weakPoints'
import { useProfileStore } from '../stores/profile'

const weakPointsStore = useWeakPointsStore()
const profileStore = useProfileStore()

const activeItems = computed(() => weakPointsStore.items.filter((wp) => wp.active))
const resolvedItems = computed(() => weakPointsStore.items.filter((wp) => !wp.active))

async function loadForCurrentProfile() {
  if (profileStore.current) {
    await weakPointsStore.load(profileStore.current.id)
  }
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
            class="rounded-lg border border-gray-200 dark:border-gray-700 px-4 py-3 text-sm"
          >
            <span class="inline-block rounded bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs px-2 py-0.5 mr-2">
              {{ wp.type }}
            </span>
            {{ wp.detail }}
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
            class="rounded-lg border border-gray-100 dark:border-gray-800 px-4 py-3 text-sm text-gray-400"
          >
            <span class="inline-block rounded bg-gray-100 dark:bg-gray-800 text-gray-400 text-xs px-2 py-0.5 mr-2">
              {{ wp.type }}
            </span>
            <span class="line-through">{{ wp.detail }}</span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
