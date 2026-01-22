<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { uploadFile } from '../api/notes';
import type { NoteType, NoteMetadata } from '../types';
import ThoughtEditor from '../components/editor/ThoughtEditor.vue';
import HarmonyEditor from '../components/editor/HarmonyEditor.vue';
import PhraseEditor from '../components/editor/PhraseEditor.vue';
import RhythmEditor from '../components/editor/RhythmEditor.vue';
import ScoreEditor from '../components/editor/ScoreEditor.vue';

const route = useRoute();
const router = useRouter();
const store = useNotesStore();

const selectedType = ref<NoteType | null>((route.query.type as NoteType) || null);
const content = ref('');
const metadata = ref<NoteMetadata>({});
const saveStatus = ref<'idle' | 'saving' | 'saved'>('idle');
const autoSaveTimer = ref<number | null>(null);
const pendingFileData = ref<{ name: string; data: number[] } | null>(null);

const noteTypes: { type: NoteType; label: string; labelRu: string; description: string; icon: string }[] = [
  { type: 'score', label: 'Original Score', labelRu: 'Партитура', description: 'Нотные записи и аранжировки', icon: '𝄞' },
  { type: 'harmony', label: 'Harmonic Study', labelRu: 'Гармония', description: 'Аккордовые последовательности', icon: '♯' },
  { type: 'phrase', label: 'Recorded Phrase', labelRu: 'Фраза', description: 'Аудиозаписи мелодий', icon: '♫' },
  { type: 'rhythm', label: 'Rhythmic Pattern', labelRu: 'Ритм', description: 'Ритмические паттерны', icon: '𝅘𝅥𝅮' },
  { type: 'thought', label: 'Personal Note', labelRu: 'Заметка', description: 'Размышления о музыке', icon: '✍' },
];

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }
});

watch(() => route.query.type, (newType) => {
  if (newType) {
    selectedType.value = newType as NoteType;
  }
});

watch([content, metadata], () => {
  if (!selectedType.value || !content.value) return;

  saveStatus.value = 'idle';

  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }

  autoSaveTimer.value = window.setTimeout(() => {
    saveNote();
  }, 2500);
}, { deep: true });

function handleGlobalKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    if (selectedType.value && !content.value) {
      selectedType.value = null;
      return;
    }
    if (content.value) {
      saveNote().then(() => {
        goBack();
      });
    } else {
      goBack();
    }
  }

  if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault();
    saveNote();
  }
}

function selectType(type: NoteType) {
  selectedType.value = type;
}

function goBack() {
  router.push({ name: 'feed' });
}

async function handleFilePath(fileName: string, fileData: number[]) {
  pendingFileData.value = { name: fileName, data: fileData };
}

async function saveNote() {
  if (!selectedType.value || !content.value) return;

  saveStatus.value = 'saving';

  try {
    if (pendingFileData.value) {
      const fileType = selectedType.value === 'phrase' ? 'audio' : 'scores';
      const path = await uploadFile(
        pendingFileData.value.name,
        pendingFileData.value.data,
        fileType
      );
      metadata.value.file_path = path;
      pendingFileData.value = null;
    }

    await store.createNote({
      note_type: selectedType.value,
      content: content.value,
      metadata: metadata.value,
    });

    saveStatus.value = 'saved';

    setTimeout(() => {
      goBack();
    }, 300);
  } catch (error) {
    console.error('Failed to save note:', error);
    saveStatus.value = 'idle';
  }
}

function handleEditorSave() {
  saveNote();
}

function updateTimeSignature(value: string) {
  metadata.value = { ...metadata.value, time_signature: value };
}

function getSelectedTypeInfo() {
  return noteTypes.find(t => t.type === selectedType.value);
}
</script>

<template>
  <div class="min-h-screen bg-[#0a0a0a] text-white font-serif overflow-x-hidden">

    <!-- Subtle Grain -->
    <div class="fixed inset-0 pointer-events-none z-50 opacity-[0.015] mix-blend-overlay bg-[url('/noise.png')]"></div>

    <!-- Ambient Light -->
    <div class="fixed inset-0 pointer-events-none z-0">
      <div class="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-amber-600/5 blur-[100px]"></div>
    </div>

    <!-- Header -->
    <header
      data-tauri-drag-region
      class="fixed top-0 left-0 right-0 z-50 bg-black/95 backdrop-blur-xl border-b border-amber-900/20"
    >
      <div class="max-w-5xl mx-auto px-6 lg:px-12 h-20 flex items-center justify-between pointer-events-none">
        <button
          @click="selectedType ? (selectedType = null) : goBack()"
          class="flex items-center gap-3 text-sm text-gray-400 hover:text-amber-400 transition-colors duration-300 pointer-events-auto group"
        >
          <span class="text-xl group-hover:-translate-x-1 transition-transform">←</span>
          <span class="font-light">{{ selectedType ? 'К выбору категории' : 'Вернуться в Архив' }}</span>
        </button>

        <div class="flex items-center gap-6 pointer-events-auto">
          <!-- Save Status -->
          <div v-if="selectedType" class="flex items-center gap-3">
            <div
              class="w-2.5 h-2.5 rounded-full transition-all duration-500"
              :class="{
                'bg-gray-600 animate-pulse': saveStatus === 'saving',
                'bg-amber-500 shadow-[0_0_8px_rgba(212,175,55,0.6)]': saveStatus === 'saved',
                'bg-white/10': saveStatus === 'idle'
              }"
            ></div>
            <span v-if="saveStatus === 'saving'" class="text-[10px] text-gray-500 uppercase tracking-widest font-sans">
              Сохранение...
            </span>
            <span v-else-if="saveStatus === 'saved'" class="text-[10px] text-amber-600 uppercase tracking-widest font-sans">
              Сохранено
            </span>
          </div>

          <!-- Manual Save Button -->
          <button
            v-if="selectedType && content"
            @click="saveNote"
            class="px-4 py-2 border border-amber-800/40 text-amber-600 hover:bg-amber-900/20 hover:border-amber-600/60 text-xs uppercase tracking-widest font-sans transition-all duration-300"
            title="Сохранить (Cmd+S)"
          >
            Сохранить
          </button>
        </div>
      </div>
    </header>

    <main class="relative z-10 max-w-5xl mx-auto pt-32 px-6 lg:px-12 pb-24">

      <!-- Type Selection -->
      <div v-if="!selectedType" class="fade-in">
        <!-- Section Header -->
        <div class="text-center mb-16">
          <div class="flex items-center justify-center gap-4 mb-8">
            <div class="h-px w-12 bg-gradient-to-r from-transparent to-amber-800/40"></div>
            <span class="text-[10px] uppercase tracking-[0.3em] text-amber-700 font-sans">Новая Запись</span>
            <div class="h-px w-12 bg-gradient-to-l from-transparent to-amber-800/40"></div>
          </div>

          <h1 class="text-4xl lg:text-5xl font-light mb-4">
            Добавить в <span class="text-amber-400">Архив</span>
          </h1>
          <p class="text-gray-500 font-light text-lg max-w-xl mx-auto">
            Выберите категорию для новой записи
          </p>
        </div>

        <!-- Type Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 max-w-4xl mx-auto">
          <button
            v-for="{ type, label, labelRu, description, icon } in noteTypes"
            :key="type"
            @click="selectType(type)"
            class="group relative"
          >
            <div class="bg-gradient-to-br from-zinc-900 to-black border border-amber-900/20 p-1 transition-all duration-500 group-hover:border-amber-600/40 group-hover:shadow-2xl group-hover:shadow-amber-900/20">
              <div class="bg-black p-8 min-h-[200px] flex flex-col items-center justify-center text-center relative overflow-hidden">
                <!-- Corner Ornaments -->
                <div class="absolute top-3 left-3 w-3 h-3 border-t border-l border-amber-900/30 group-hover:border-amber-600/50 transition-colors"></div>
                <div class="absolute top-3 right-3 w-3 h-3 border-t border-r border-amber-900/30 group-hover:border-amber-600/50 transition-colors"></div>
                <div class="absolute bottom-3 left-3 w-3 h-3 border-b border-l border-amber-900/30 group-hover:border-amber-600/50 transition-colors"></div>
                <div class="absolute bottom-3 right-3 w-3 h-3 border-b border-r border-amber-900/30 group-hover:border-amber-600/50 transition-colors"></div>

                <!-- Icon -->
                <div class="text-5xl mb-6 text-amber-800/40 group-hover:text-amber-500 transition-colors duration-500">
                  {{ icon }}
                </div>

                <!-- Labels -->
                <h3 class="text-xl font-light text-white mb-1 group-hover:text-amber-400 transition-colors">
                  {{ labelRu }}
                </h3>
                <div class="text-[10px] uppercase tracking-[0.2em] text-amber-800/60 mb-3 font-sans">
                  {{ label }}
                </div>
                <p class="text-xs text-gray-600 group-hover:text-gray-500 transition-colors">
                  {{ description }}
                </p>
              </div>
            </div>
          </button>
        </div>

        <!-- Keyboard Hint -->
        <div class="text-center mt-12">
          <div class="inline-flex items-center gap-3 text-[10px] text-gray-700 font-sans">
            <kbd class="px-2 py-1 bg-zinc-900 border border-zinc-800 rounded text-gray-500 tracking-wider">ESC</kbd>
            <span>для возврата</span>
          </div>
        </div>
      </div>

      <!-- Editor -->
      <div v-else class="fade-in">
        <!-- Editor Header -->
        <div class="mb-12">
          <div class="flex items-center gap-4 mb-6">
            <div class="h-px flex-1 bg-gradient-to-r from-transparent to-amber-800/30"></div>
            <div class="flex items-center gap-4 text-[10px] uppercase tracking-[0.3em] text-amber-700 font-sans">
              <span>{{ getSelectedTypeInfo()?.label }}</span>
            </div>
            <div class="h-px flex-1 bg-gradient-to-l from-transparent to-amber-800/30"></div>
          </div>

          <div class="text-center">
            <div class="text-4xl text-amber-500/60 mb-4">
              {{ getSelectedTypeInfo()?.icon }}
            </div>
            <h2 class="text-3xl lg:text-4xl font-light">
              Новая {{ getSelectedTypeInfo()?.labelRu }}
            </h2>
          </div>
        </div>

        <!-- Editor Frame -->
        <div class="relative bg-gradient-to-br from-zinc-900 to-black border border-amber-900/30 p-1.5 shadow-2xl max-w-3xl mx-auto">
          <div class="bg-black relative">
            <!-- Corner Details -->
            <div class="absolute top-4 left-4 w-5 h-5 border-t-2 border-l-2 border-amber-800/30"></div>
            <div class="absolute top-4 right-4 w-5 h-5 border-t-2 border-r-2 border-amber-800/30"></div>
            <div class="absolute bottom-4 left-4 w-5 h-5 border-b-2 border-l-2 border-amber-800/30"></div>
            <div class="absolute bottom-4 right-4 w-5 h-5 border-b-2 border-r-2 border-amber-800/30"></div>

            <div class="p-8 lg:p-12">
              <component
                :is="selectedType === 'thought' ? ThoughtEditor :
                     selectedType === 'harmony' ? HarmonyEditor :
                     selectedType === 'phrase' ? PhraseEditor :
                     selectedType === 'rhythm' ? RhythmEditor :
                     ScoreEditor"
                v-model="content"
                :time-signature="metadata.time_signature"
                :file-path="metadata.file_path"
                @save="handleEditorSave"
                @update:time-signature="updateTimeSignature"
                @update:file-path="handleFilePath"
              />
            </div>

            <!-- Footer -->
            <div class="px-8 lg:px-12 pb-6">
              <div class="pt-6 border-t border-amber-900/10 flex justify-between items-center text-[10px] uppercase tracking-[0.2em] text-gray-700 font-sans">
                <span>Архив Gmazz</span>
                <div class="flex items-center gap-4">
                  <span class="hidden sm:inline">Cmd+S для сохранения</span>
                  <span>{{ new Date().toLocaleDateString('ru-RU') }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.fade-in {
  animation: fadeIn 0.8s ease-out forwards;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
