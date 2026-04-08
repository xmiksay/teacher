<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useVocabStore } from '../stores/vocab'
import { useProfileStore } from '../stores/profile'

const vocabStore = useVocabStore()
const profileStore = useProfileStore()

const showAnswer = ref(false)
const newWord = ref('')
const newTranslation = ref('')
const tab = ref<'flashcard' | 'list'>('flashcard')

const currentCard = computed(() => {
  if (vocabStore.words.length === 0) return null
  return vocabStore.words[vocabStore.flashcardIndex]
})

async function loadForCurrentProfile() {
  if (profileStore.current) {
    await vocabStore.loadVocab(profileStore.current.id)
  }
}

onMounted(loadForCurrentProfile)

watch(() => profileStore.current?.id, loadForCurrentProfile)

function flip() {
  showAnswer.value = !showAnswer.value
}

function next() {
  showAnswer.value = false
  vocabStore.nextFlashcard()
}

async function addWord() {
  if (!profileStore.current || !newWord.value.trim() || !newTranslation.value.trim()) return
  await vocabStore.addWord({
    profile_id: profileStore.current.id,
    word: newWord.value.trim(),
    translation: newTranslation.value.trim(),
  })
  newWord.value = ''
  newTranslation.value = ''
}
</script>

<template>
  <div class="p-4">
    <div v-if="!profileStore.current" class="text-center text-gray-500 mt-8">
      <p>No profile selected. Go to <router-link to="/settings" class="text-blue-600 underline">Settings</router-link>.</p>
    </div>

    <template v-else>
      <!-- Tabs -->
      <div class="flex gap-4 mb-6 border-b border-gray-200 dark:border-gray-700">
        <button
          @click="tab = 'flashcard'"
          :class="['pb-2 text-sm font-medium', tab === 'flashcard' ? 'border-b-2 border-blue-600 text-blue-600' : 'text-gray-500']"
        >
          Flashcards
        </button>
        <button
          @click="tab = 'list'"
          :class="['pb-2 text-sm font-medium', tab === 'list' ? 'border-b-2 border-blue-600 text-blue-600' : 'text-gray-500']"
        >
          Word List
        </button>
      </div>

      <!-- Flashcard mode -->
      <div v-if="tab === 'flashcard'" class="flex flex-col items-center gap-4">
        <div v-if="!currentCard" class="text-gray-400 mt-12">
          No vocabulary words yet. Add some below or start a lesson.
        </div>
        <div
          v-else
          @click="flip"
          class="w-80 h-48 flex items-center justify-center rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 cursor-pointer select-none shadow-sm hover:shadow-md transition-shadow"
        >
          <div class="text-center px-4">
            <p class="text-2xl font-medium">{{ showAnswer ? currentCard.translation : currentCard.word }}</p>
            <p class="text-xs text-gray-400 mt-2">{{ showAnswer ? 'Translation' : 'Click to reveal' }}</p>
            <p v-if="showAnswer && currentCard.context" class="text-xs text-gray-500 mt-1 italic">
              "{{ currentCard.context }}"
            </p>
          </div>
        </div>
        <div class="flex gap-2">
          <button @click="next" class="rounded-lg bg-gray-200 dark:bg-gray-700 px-4 py-2 text-sm">
            Next
          </button>
        </div>
        <p class="text-xs text-gray-400">
          {{ vocabStore.flashcardIndex + 1 }} / {{ vocabStore.words.length }}
        </p>
      </div>

      <!-- Word list mode -->
      <div v-if="tab === 'list'">
        <!-- Add word form -->
        <form @submit.prevent="addWord" class="flex gap-2 mb-4">
          <input
            v-model="newWord"
            placeholder="Word"
            class="flex-1 rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm"
          />
          <input
            v-model="newTranslation"
            placeholder="Translation"
            class="flex-1 rounded-lg border border-gray-300 dark:border-gray-600 bg-transparent px-3 py-2 text-sm"
          />
          <button type="submit" class="rounded-lg bg-blue-600 px-4 py-2 text-sm text-white">Add</button>
        </form>

        <div v-if="vocabStore.words.length === 0" class="text-gray-400 text-center mt-8">
          No vocabulary words yet.
        </div>

        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-gray-200 dark:border-gray-700 text-left text-gray-500">
              <th class="py-2">Word</th>
              <th class="py-2">Translation</th>
              <th class="py-2">Added by</th>
              <th class="py-2">Errors</th>
              <th class="py-2"></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="word in vocabStore.words"
              :key="word.id"
              class="border-b border-gray-100 dark:border-gray-800"
            >
              <td class="py-2">{{ word.word }}</td>
              <td class="py-2">{{ word.translation }}</td>
              <td class="py-2">
                <span :class="word.added_by === 'claude' ? 'text-purple-500' : 'text-gray-500'">
                  {{ word.added_by }}
                </span>
              </td>
              <td class="py-2">{{ word.error_count }}</td>
              <td class="py-2">
                <button
                  @click="vocabStore.deleteWord(word.id)"
                  class="text-red-500 hover:text-red-700 text-xs"
                >
                  Delete
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>
