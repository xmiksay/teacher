<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { useProfileStore } from './stores/profile'

const authStore = useAuthStore()
const profileStore = useProfileStore()
const router = useRouter()

onMounted(async () => {
  if (authStore.isLoggedIn) {
    await profileStore.loadProfiles()
  }
})

function logout() {
  authStore.logout()
  router.push('/login')
}
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
    <nav v-if="authStore.isLoggedIn" class="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
      <div class="max-w-5xl mx-auto px-4 flex items-center h-14 gap-6">
        <span class="font-semibold text-lg">Teacher</span>
        <router-link
          to="/lesson"
          class="text-sm hover:text-blue-600 dark:hover:text-blue-400"
          active-class="text-blue-600 dark:text-blue-400 font-medium"
        >
          Lesson
        </router-link>
        <router-link
          to="/lesson-history"
          class="text-sm hover:text-blue-600 dark:hover:text-blue-400"
          active-class="text-blue-600 dark:text-blue-400 font-medium"
        >
          History
        </router-link>
        <router-link
          to="/vocab"
          class="text-sm hover:text-blue-600 dark:hover:text-blue-400"
          active-class="text-blue-600 dark:text-blue-400 font-medium"
        >
          Vocabulary
        </router-link>
        <router-link
          to="/weak-points"
          class="text-sm hover:text-blue-600 dark:hover:text-blue-400"
          active-class="text-blue-600 dark:text-blue-400 font-medium"
        >
          Weak Points
        </router-link>
        <router-link
          to="/settings"
          class="text-sm hover:text-blue-600 dark:hover:text-blue-400"
          active-class="text-blue-600 dark:text-blue-400 font-medium"
        >
          Settings
        </router-link>
        <div class="ml-auto flex items-center gap-4">
          <select
            v-if="profileStore.profiles.length > 0"
            :value="profileStore.current?.id ?? ''"
            @change="profileStore.current = profileStore.profiles.find((p) => p.id === ($event.target as HTMLSelectElement).value) ?? null"
            class="rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-2 py-1 text-sm"
          >
            <option v-for="profile in profileStore.profiles" :key="profile.id" :value="profile.id">
              {{ profile.language }} ({{ profile.level }})
            </option>
          </select>
          <button
            @click="logout"
            class="text-sm text-gray-500 hover:text-red-500"
          >
            Log out
          </button>
        </div>
      </div>
    </nav>
    <main class="max-w-5xl mx-auto">
      <router-view />
    </main>
  </div>
</template>
