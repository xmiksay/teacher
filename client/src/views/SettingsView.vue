<script setup lang="ts">
import { ref, watch } from 'vue'
import { useProfileStore } from '../stores/profile'
import { useVocabStore } from '../stores/vocab'

const profileStore = useProfileStore()
const vocabStore = useVocabStore()

const newLanguage = ref('')
const newLevel = ref('A1')

const editLevel = ref('')
const editStyle = ref('')
const editExplanationLang = ref('')
const editPersonalNote = ref('')

watch(
  () => profileStore.current,
  (profile) => {
    if (profile) {
      editLevel.value = profile.level
      editStyle.value = profile.style
      editExplanationLang.value = profile.explanation_language
      editPersonalNote.value = profile.personal_note
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
    personal_note: editPersonalNote.value,
  })
}

function selectProfile(profile: typeof profileStore.current) {
  profileStore.current = profile
}

async function clearVocabulary() {
  if (!profileStore.current) return
  if (!confirm(`Delete all vocabulary for ${profileStore.current.language}? This cannot be undone.`)) return
  await vocabStore.deleteAll(profileStore.current.id)
}

async function deleteProfile() {
  if (!profileStore.current) return
  if (!confirm(`Delete ${profileStore.current.language} profile and all its lessons, vocabulary, and weak points? This cannot be undone.`)) return
  await profileStore.deleteProfile(profileStore.current.id)
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
        <select v-model="newLevel" class="rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 px-3 py-2 text-sm">
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
          <select v-model="editLevel" class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 px-3 py-2 text-sm">
            <option v-for="l in ['A1','A2','B1','B2','C1','C2']" :key="l" :value="l">{{ l }}</option>
          </select>
        </div>
        <div>
          <label class="block text-xs text-gray-500 mb-1">Tutor Style</label>
          <select v-model="editStyle" class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 px-3 py-2 text-sm">
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
        <div>
          <label class="block text-xs text-gray-500 mb-1">Personal Note</label>
          <textarea
            v-model="editPersonalNote"
            placeholder="Describe how you prefer to be taught, e.g. 'Focus on conversational practice, use lots of examples, correct me immediately...'"
            rows="4"
            class="w-full rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm"
          />
          <p class="text-xs text-gray-400 mt-1">This note will be included in the tutor's instructions to personalize your learning experience.</p>
        </div>
        <button type="submit" class="rounded-lg bg-blue-600 px-6 py-2 text-sm text-white">
          Save
        </button>
      </form>

      <!-- Danger zone -->
      <div class="mt-8 border-t border-gray-200 dark:border-gray-700 pt-6">
        <h4 class="text-sm font-medium text-red-600 dark:text-red-400 mb-3">Danger Zone</h4>
        <div class="space-y-3">
          <div>
            <button
              @click="clearVocabulary"
              class="rounded-lg border border-red-300 dark:border-red-700 px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
            >
              Clear all vocabulary
            </button>
            <p class="text-xs text-gray-400 mt-1">Permanently removes all vocabulary words for this profile.</p>
          </div>
          <div>
            <button
              @click="deleteProfile"
              class="rounded-lg border border-red-300 dark:border-red-700 px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
            >
              Delete profile
            </button>
            <p class="text-xs text-gray-400 mt-1">Permanently deletes this profile and all its lessons, vocabulary, and weak points.</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
