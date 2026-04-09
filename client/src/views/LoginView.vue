<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { useProfileStore } from '../stores/profile'

const authStore = useAuthStore()
const profileStore = useProfileStore()
const router = useRouter()

const isRegister = ref(false)
const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function submit() {
  error.value = ''
  if (!username.value.trim() || !password.value) {
    error.value = 'Username and password are required'
    return
  }
  loading.value = true
  try {
    if (isRegister.value) {
      await authStore.register(username.value.trim(), password.value)
    } else {
      await authStore.login(username.value.trim(), password.value)
    }
    await profileStore.loadProfiles()
    router.push('/lesson')
  } catch (e: any) {
    error.value = e.message || 'Something went wrong'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
    <div class="w-full max-w-sm p-6">
      <h1 class="text-2xl font-semibold text-center mb-8 text-gray-900 dark:text-gray-100">
        Teacher
      </h1>

      <form @submit.prevent="submit" class="space-y-4">
        <div>
          <label class="block text-xs text-gray-500 mb-1">Username</label>
          <input
            v-model="username"
            type="text"
            autocomplete="username"
            class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-gray-500 mb-1">Password</label>
          <input
            v-model="password"
            type="password"
            autocomplete="current-password"
            class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div v-if="error" class="text-red-500 text-sm">{{ error }}</div>

        <button
          type="submit"
          :disabled="loading"
          class="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {{ loading ? '...' : isRegister ? 'Register' : 'Log in' }}
        </button>
      </form>

      <p class="mt-4 text-center text-sm text-gray-500">
        <button
          @click="isRegister = !isRegister; error = ''"
          class="text-blue-600 dark:text-blue-400 hover:underline"
        >
          {{ isRegister ? 'Already have an account? Log in' : "Don't have an account? Register" }}
        </button>
      </p>
    </div>
  </div>
</template>
