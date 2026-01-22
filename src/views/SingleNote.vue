<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { getAssetPath } from '../api/notes';
import Waveform from '../components/common/Waveform.vue';

const route = useRoute();
const router = useRouter();
const store = useNotesStore();
const isPlaying = ref(false);
const audio = ref<HTMLAudioElement | null>(null);
const imagePath = ref<string>('');
const imageScale = ref(1);

const note = computed(() => store.currentNote);

const typeInfo: Record<string, { label: string; labelRu: string; icon: string; }> = {
  thought: { label: 'Personal Reflection', labelRu: 'Личная Заметка', icon: '✍' },
  harmony: { label: 'Harmonic Analysis', labelRu: 'Гармонический Анализ', icon: '♯' },
  phrase: { label: 'Recorded Phrase', labelRu: 'Запись Фразы', icon: '♫' },
  rhythm: { label: 'Rhythmic Pattern', labelRu: 'Ритмический Паттерн', icon: '♩' },
  score: { label: 'Original Manuscript', labelRu: 'Оригинальная Партитура', icon: '𝄞' },
};

onMounted(async () => {
  const id = route.params.id as string;
  await store.fetchNote(id);

  if (note.value?.note_type === 'phrase' && note.value.metadata.file_path) {
    const path = await getAssetPath(note.value.metadata.file_path);
    audio.value = new Audio(path);
    audio.value.addEventListener('ended', () => {
      isPlaying.value = false;
    });
  }

  if (note.value?.note_type === 'score' && note.value.metadata.file_path) {
    imagePath.value = await getAssetPath(note.value.metadata.file_path);
  }

  document.addEventListener('keydown', handleKeydown);
  window.addEventListener('wheel', handleWheel, { passive: false });
});

onUnmounted(() => {
  if (audio.value) {
    audio.value.pause();
    audio.value = null;
  }
  store.clearCurrentNote();
  document.removeEventListener('keydown', handleKeydown);
  window.removeEventListener('wheel', handleWheel);
});

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    goBack();
  }
}

function goBack() {
  router.push({ name: 'feed' });
}

function togglePlay() {
  if (!audio.value) return;

  if (isPlaying.value) {
    audio.value.pause();
  } else {
    audio.value.play();
  }
  isPlaying.value = !isPlaying.value;
}

async function handleDelete() {
  if (!note.value) return;

  const confirmed = confirm("Удалить эту работу из архива? Это действие нельзя отменить.");
  if (confirmed) {
    const success = await store.deleteNote(note.value.id);
    if (success) {
      router.push({ name: 'feed' });
    }
  }
}

function handleWheel(event: WheelEvent) {
  if (note.value?.note_type !== 'score') return;
  if (event.ctrlKey || event.metaKey) {
    event.preventDefault();
    const delta = event.deltaY > 0 ? -0.1 : 0.1;
    imageScale.value = Math.max(0.5, Math.min(4, imageScale.value + delta));
  }
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  const months = ['Января', 'Февраля', 'Марта', 'Апреля', 'Мая', 'Июня',
    'Июля', 'Августа', 'Сентября', 'Октября', 'Ноября', 'Декабря'];
  return `${date.getDate()} ${months[date.getMonth()]} ${date.getFullYear()}`;
}
</script>

<template>
  <div class="min-h-screen bg-[#FAF7F2] text-[#3D3428] font-serif overflow-x-hidden">

    <!-- Header -->
    <header
        data-tauri-drag-region
        class="fixed top-0 left-0 right-0 z-50 bg-[#FAF7F2]/95 backdrop-blur-md border-b border-[#D4CAB5]"
    >
      <div class="max-w-5xl mx-auto px-6 lg:px-12 h-20 flex items-center justify-between pointer-events-none">
        <button
            @click="goBack"
            class="flex items-center gap-3 text-sm text-[#8B7E6A] hover:text-[#A67C00] transition-colors duration-300 pointer-events-auto group"
        >
          <span class="text-xl group-hover:-translate-x-1 transition-transform">←</span>
          <span class="font-light">Вернуться в Архив</span>
        </button>

        <div class="flex items-center gap-6 pointer-events-auto">
          <button
              v-if="note"
              @click="handleDelete"
              class="text-[10px] uppercase tracking-[0.25em] text-[#8B7E6A] hover:text-red-600 transition-colors font-sans"
          >
            Удалить
          </button>

          <div class="hidden md:flex items-center gap-2 text-xs text-[#8B7E6A] font-sans">
            <kbd class="px-2 py-1 bg-[#F5F1E8] border border-[#D4CAB5] rounded text-[10px] text-[#8B7E6A] tracking-wider">ESC</kbd>
            <span class="text-[#A89F8B]">для выхода</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Exhibition -->
    <main class="relative z-10 max-w-4xl mx-auto px-6 lg:px-12 pt-32 pb-24">
      <Transition
          enter-active-class="transition-all duration-700 ease-out"
          leave-active-class="transition-all duration-300 ease-in"
          enter-from-class="opacity-0 translate-y-8"
          leave-to-class="opacity-0 -translate-y-8"
          mode="out-in"
      >
        <div v-if="note" :key="note.id" class="space-y-12">

          <!-- Exhibit Label -->
          <div class="space-y-8">
            <!-- Catalog Info -->
            <div class="flex items-center gap-4">
              <div class="h-px flex-1 bg-gradient-to-r from-transparent to-[#D4CAB5]"></div>
              <div class="flex items-center gap-4 text-[10px] uppercase tracking-[0.3em] text-[#A67C00] font-sans">
                <span>№ {{ note.id.substring(0, 8) }}</span>
                <span class="text-[#D4CAB5]">•</span>
                <span>{{ typeInfo[note.note_type]?.label }}</span>
              </div>
              <div class="h-px flex-1 bg-gradient-to-l from-transparent to-[#D4CAB5]"></div>
            </div>

            <!-- Title Block -->
            <div class="text-center space-y-4">
              <div class="text-5xl text-[#A67C00]/60">
                {{ typeInfo[note.note_type]?.icon }}
              </div>
              <h1 class="text-4xl lg:text-5xl font-light tracking-tight text-[#3D3428]">
                {{ typeInfo[note.note_type]?.labelRu }}
              </h1>
              <p class="text-lg text-[#8B7E6A] font-light">
                {{ formatDate(note.created_at) }}
              </p>
            </div>
          </div>

          <!-- Exhibition Frame -->
          <article class="relative">
            <div class="relative bg-white border border-[#E0D9C8] rounded-lg p-1.5 shadow-lg">

              <!-- Inner Mat -->
              <div class="bg-[#FFFEFA] rounded relative overflow-hidden">

                <!-- Corner Details -->
                <div class="absolute top-4 left-4 w-6 h-6 border-t-2 border-l-2 border-[#D4CAB5]"></div>
                <div class="absolute top-4 right-4 w-6 h-6 border-t-2 border-r-2 border-[#D4CAB5]"></div>
                <div class="absolute bottom-4 left-4 w-6 h-6 border-b-2 border-l-2 border-[#D4CAB5]"></div>
                <div class="absolute bottom-4 right-4 w-6 h-6 border-b-2 border-r-2 border-[#D4CAB5]"></div>

                <div class="p-8 lg:p-12 relative z-10">

                  <!-- Thought (Personal Notes) -->
                  <template v-if="note.note_type === 'thought'">
                    <div class="max-w-2xl mx-auto space-y-8">
                      <div class="w-16 h-1 bg-[#A67C00]/40 mx-auto"></div>
                      <div class="text-xl lg:text-2xl leading-relaxed text-[#3D3428] whitespace-pre-line text-center font-light">
                        {{ note.content }}
                      </div>
                      <div class="flex justify-center pt-6">
                        <div class="text-3xl text-[#A67C00]/40">✍</div>
                      </div>
                    </div>
                  </template>

                  <!-- Harmony (Theory) -->
                  <template v-else-if="note.note_type === 'harmony'">
                    <div class="space-y-6">
                      <div class="text-center mb-8">
                        <h3 class="text-sm uppercase tracking-[0.3em] text-[#A67C00] font-sans mb-2">
                          Гармонический Анализ
                        </h3>
                      </div>

                      <div class="bg-[#F5F1E8] border border-[#D4CAB5] rounded-lg p-8 overflow-x-auto custom-scrollbar">
                        <pre class="font-mono text-base lg:text-lg text-[#8B5A2B] whitespace-pre-wrap leading-loose">{{ note.content }}</pre>
                      </div>
                    </div>
                  </template>

                  <!-- Phrase (Audio Recording) -->
                  <template v-else-if="note.note_type === 'phrase'">
                    <div class="space-y-10">

                      <!-- Playback Console -->
                      <div class="bg-[#F5F1E8] border border-[#D4CAB5] rounded-lg p-8 space-y-6">
                        <div class="flex items-center justify-center gap-8">
                          <!-- Play Button -->
                          <button
                              @click="togglePlay"
                              class="w-20 h-20 flex items-center justify-center rounded-full bg-gradient-to-br from-[#A67C00] to-[#B8860B] text-white hover:from-[#B8860B] hover:to-[#C9A227] transition-all duration-300 shadow-lg shadow-[#A67C00]/30 relative group"
                          >
                            <svg v-if="!isPlaying" class="w-8 h-8 ml-1" fill="currentColor" viewBox="0 0 24 24">
                              <path d="M8 5v14l11-7z" />
                            </svg>
                            <svg v-else class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                            </svg>

                            <!-- Pulse Effect When Playing -->
                            <div v-if="isPlaying" class="absolute inset-0 rounded-full bg-[#C9A227] animate-ping opacity-20"></div>
                          </button>
                        </div>

                        <!-- Waveform Display -->
                        <div class="h-24 w-full bg-white border border-[#D4CAB5] rounded-lg relative overflow-hidden">
                          <div class="absolute left-1/2 top-0 bottom-0 w-0.5 bg-[#A67C00] z-20"></div>
                          <Waveform class="w-full h-full opacity-60" />
                        </div>

                        <div class="text-center">
                          <div class="text-xs uppercase tracking-widest text-[#8B7E6A] font-sans">
                            {{ isPlaying ? 'Воспроизведение...' : 'Готов к воспроизведению' }}
                          </div>
                        </div>
                      </div>

                      <!-- Annotation -->
                      <div v-if="note.metadata.comment || note.content" class="border-l-2 border-[#A67C00]/40 pl-8">
                        <p class="text-lg lg:text-xl text-[#5C5245] italic leading-relaxed">
                          "{{ note.metadata.comment || note.content }}"
                        </p>
                      </div>
                    </div>
                  </template>

                  <!-- Rhythm -->
                  <template v-else-if="note.note_type === 'rhythm'">
                    <div class="max-w-2xl mx-auto space-y-10">

                      <div class="text-center">
                        <h3 class="text-sm uppercase tracking-[0.3em] text-[#A67C00] font-sans mb-8">
                          Ритмическая Структура
                        </h3>
                      </div>

                      <!-- Time Signature Display -->
                      <div class="flex justify-center">
                        <div class="relative">
                          <div class="w-32 h-32 bg-white border-2 border-[#D4CAB5] rounded-lg flex flex-col items-center justify-center shadow-sm">
                            <span class="text-5xl font-light text-[#A67C00] leading-none">
                              {{ note.metadata.time_signature?.split('/')[0] || '4' }}
                            </span>
                            <div class="w-12 h-px bg-[#D4CAB5] my-2"></div>
                            <span class="text-5xl font-light text-[#5C5245] leading-none">
                              {{ note.metadata.time_signature?.split('/')[1] || '4' }}
                            </span>
                          </div>

                          <!-- Corner Accents -->
                          <div class="absolute -top-1 -left-1 w-4 h-4 border-t-2 border-l-2 border-[#A67C00]"></div>
                          <div class="absolute -top-1 -right-1 w-4 h-4 border-t-2 border-r-2 border-[#A67C00]"></div>
                          <div class="absolute -bottom-1 -left-1 w-4 h-4 border-b-2 border-l-2 border-[#A67C00]"></div>
                          <div class="absolute -bottom-1 -right-1 w-4 h-4 border-b-2 border-r-2 border-[#A67C00]"></div>
                        </div>
                      </div>

                      <!-- Pattern -->
                      <div class="bg-[#F5F1E8] border border-[#D4CAB5] rounded-lg p-8 text-center">
                        <div class="text-xs uppercase tracking-widest text-[#8B7E6A] mb-4 font-sans">
                          Groove Pattern
                        </div>
                        <div class="font-mono text-xl lg:text-2xl text-[#3D3428] tracking-wider">
                          {{ note.content }}
                        </div>
                      </div>
                    </div>
                  </template>

                  <!-- Score (Sheet Music) -->
                  <template v-else-if="note.note_type === 'score'">
                    <div class="space-y-6">

                      <div class="text-center mb-6">
                        <h3 class="text-sm uppercase tracking-[0.3em] text-[#A67C00] font-sans">
                          Оригинальная Партитура
                        </h3>
                      </div>

                      <!-- Score Viewer -->
                      <div class="relative bg-[#F5F1E8] p-3 rounded-lg border border-[#D4CAB5]">
                        <div class="bg-white overflow-auto max-h-[70vh] flex items-center justify-center p-6 rounded"
                             :class="imageScale > 1 ? 'cursor-zoom-out' : 'cursor-zoom-in'">
                          <img
                              v-if="imagePath"
                              :src="imagePath"
                              alt="Musical Score"
                              :style="{ transform: `scale(${imageScale})` }"
                              class="max-w-full shadow-lg transition-transform duration-300 ease-out origin-center"
                          />
                          <div v-else class="h-64 flex items-center justify-center text-[#8B7E6A] font-sans">
                            <div class="animate-pulse">Загрузка партитуры...</div>
                          </div>
                        </div>
                      </div>

                      <!-- Score Info -->
                      <div class="flex justify-between items-center text-[10px] uppercase tracking-wider text-[#8B7E6A] font-sans px-2">
                        <span class="text-[#5C5245]">{{ note.content }}</span>
                        <span class="text-[#A67C00]">Ctrl + Scroll для масштабирования</span>
                      </div>
                    </div>
                  </template>
                </div>

                <!-- Signature Stamp -->
                <div class="px-8 lg:px-12 pb-8">
                  <div class="pt-8 border-t border-[#E0D9C8] flex justify-between items-center text-[10px] uppercase tracking-[0.25em] text-[#8B7E6A] font-sans">
                    <span>Архив Gmazz</span>
                    <span class="text-[#A67C00]">{{ new Date(note.created_at).getFullYear() }}</span>
                  </div>
                </div>
              </div>
            </div>
          </article>

        </div>

        <!-- Loading State -->
        <div v-else class="min-h-[60vh] flex flex-col items-center justify-center space-y-6">
          <div class="text-5xl text-[#A67C00]/50 animate-pulse">𝄞</div>
          <p class="text-sm uppercase tracking-[0.3em] text-[#8B7E6A] font-sans">
            Извлечение из архива...
          </p>
        </div>
      </Transition>
    </main>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  height: 8px;
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: #F5F1E8;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #D4CAB5;
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #A67C00;
}
</style>