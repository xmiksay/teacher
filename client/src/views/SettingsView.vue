<script setup lang="ts">
import { ref, watch } from 'vue'
import { useProfileStore } from '../stores/profile'

const profileStore = useProfileStore()

const newLanguage = ref('')
const newLevel = ref('A1')

const editLevel = ref('')
const editStyle = ref('')
const editExplanationLang = ref('')

watch(
  () => profileStore.current,
  (profile) => {
    if (profile) {
      editLevel.value = profile.level
      editStyle.value = profile.style
      editExplanationLang.value = profile.explanation_language
    }
  },
  { immediate: true }
)

async function createProfile() {
  if (!newLanguage.value.trim()) return
  await profileStore.createProfile({
    language: newLanguage.value.trim(),
    level: newLevel.value,
  })
  newLanguage.value = ''
}

async function saveSettings() {
  if (!profileStore.current) return
  await profileStore.updateProfile(profileStore.current.id, {
    level: editLevel.value,
    style: editStyle.value,
    explanation_language: editExplanationLang.value,
  })
}

function selectProfile(profile: typeof profileStore.current) {
  profileStore.current = profile
}
</script>

<template>
  <div class="p-4 max-w-lg mx-auto">
    <h2 class="text-xl font-semibold mb-6">Settings</h2>

    <!-- Create new profile -->
    <div class="mb-8">
      <h3 class="text-sm font-medium text-gray-500 mb-3">New Language Profile</h3>
      <form @submit.prevent="createProfile" class="flex gap-2">
        <input
          v-model="newLanguage"
          placeholder="Language (e.g. Spanish)"
          class="flex-1 rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm"
        />
        <select v-model="newLevel" class="rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm">
          <option v-for="l in ['A1','A2','B1','B2','C1','C2']" :key="l" :value="l">{{ l }}</option>
        </select>
        <button type="submit" class="rounded-lg bg-blue-600 px-4 py-2 text-sm text-white">Create</button>
      </form>
    </div>

    <!-- Profile list -->
    <div class="mb-8">
      <h3 class="text-sm font-medium text-gray-500 mb-3">Your Profiles</h3>
      <div v-if="profileStore.profiles.length === 0" class="text-gray-400 text-sm">
        No profiles yet. Create one above.
      </div>
      <div class="space-y-2">
        <button
          v-for="profile in profileStore.profiles"
          :key="profile.id"
          @click="selectProfile(profile)"
          :class="[
            'w-full text-left rounded-lg border px-4 py-3 text-sm transition-colors',
            profileStore.current?.id === profile.id
              ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
              : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800'
          ]"
        >
          <span class="font-medium">{{ profile.language }}</span>
          <span class="text-gray-500 ml-2">{{ profile.level }}</span>
        </button>
      </div>
    </div>

    <!-- Edit current profile -->
    <div v-if="profileStore.current">
      <h3 class="text-sm font-medium text-gray-500 mb-3">
        Edit: {{ profileStore.current.language }}
      </h3>
      <form @submit.prevent="saveSettings" class="space-y-3">
        <div>
          <label class="block text-xs text-gray-500 mb-1">Level</label>
          <select v-model="editLevel" class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm">
            <option v-for="l in ['A1','A2','B1','B2','C1','C2']" :key="l" :value="l">{{ l }}</option>
          </select>
        </div>
        <div>
          <label class="block text-xs text-gray-500 mb-1">Tutor Style</label>
          <select v-model="editStyle" class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm">
            <option value="friendly">Friendly</option>
            <option value="strict">Strict</option>
            <option value="immersive">Immersive</option>
            <option value="encouraging">Encouraging</option>
          </select>
        </div>
        <div>
          <label class="block text-xs text-gray-500 mb-1">Explanation Language</label>
          <input
            v-model="editExplanationLang"
            placeholder="en, cs, target..."
            class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm"
          />
        </div>
        <button type="submit" class="rounded-lg bg-blue-600 px-6 py-2 text-sm text-white">
          Save
        </button>
      </form>
    </div>
  </div>
</template>
