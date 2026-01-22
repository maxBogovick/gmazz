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

// Конфигурация типов для стиля (цвета и иконки)
const typeInfo: Record<string, { label: string; gradient: string; icon: string; bgColor: string; textColor: string }> = {
  thought: {
    label: 'Thought',
    gradient: 'from-purple-400 via-pink-400 to-rose-400',
    bgColor: 'bg-purple-500/10',
    textColor: 'text-purple-400',
    icon: '✨'
  },
  harmony: {
    label: 'Harmony',
    gradient: 'from-amber-400 via-yellow-500 to-orange-500',
    bgColor: 'bg-amber-500/10',
    textColor: 'text-amber-400',
    icon: '🎵'
  },
  phrase: {
    label: 'Phrase',
    gradient: 'from-emerald-400 via-teal-400 to-cyan-500',
    bgColor: 'bg-emerald-500/10',
    textColor: 'text-emerald-400',
    icon: '📝'
  },
  rhythm: {
    label: 'Rhythm',
    gradient: 'from-fuchsia-400 via-purple-500 to-violet-600',
    bgColor: 'bg-fuchsia-500/10',
    textColor: 'text-fuchsia-400',
    icon: '⚡'
  },
  score: {
    label: 'Score',
    gradient: 'from-blue-400 via-indigo-500 to-purple-600',
    bgColor: 'bg-blue-500/10',
    textColor: 'text-blue-400',
    icon: '🎼'
  },
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
});

onUnmounted(() => {
  if (audio.value) {
    audio.value.pause();
    audio.value = null;
  }
  store.clearCurrentNote();
  document.removeEventListener('keydown', handleKeydown);
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
  return date.toLocaleDateString('en-US', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}
</script>

<template>
  <div class="min-h-screen relative bg-slate-950 overflow-x-hidden">
    <!-- Animated Background Orbs -->
    <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden">
      <div class="absolute w-[600px] h-[600px] rounded-full blur-[120px] opacity-20 -top-48 -right-24 bg-gradient-radial from-amber-500/60 to-transparent animate-float"></div>
      <div class="absolute w-[500px] h-[500px] rounded-full blur-[120px] opacity-15 bottom-24 -left-24 bg-gradient-radial from-orange-500/60 to-transparent animate-float-reverse"></div>
    </div>

    <!-- Header -->
    <header class="fixed top-0 left-0 right-0 z-50 bg-slate-950/80 backdrop-blur-2xl border-b border-slate-800/50">
      <div class="max-w-5xl mx-auto px-6 lg:px-12 h-20 flex items-center justify-between">
        <button
            @click="goBack"
            class="group flex items-center gap-3 text-sm font-medium text-slate-400 hover:text-slate-100 transition-all duration-300"
        >
          <div class="w-10 h-10 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center transition-all duration-300 group-hover:border-amber-500/50 group-hover:bg-slate-800 group-hover:-translate-x-1">
            <svg class="w-5 h-5 transition-transform duration-300 group-hover:-translate-x-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </div>
          <span class="tracking-wide hidden sm:inline">Back to Notebook</span>
        </button>

        <!-- ESC hint -->
        <div class="flex items-center gap-2 text-xs text-slate-600 font-medium">
          <span class="hidden md:inline">Press</span>
          <kbd class="px-2.5 py-1.5 bg-slate-900 border border-slate-800 rounded-lg text-slate-400 font-mono font-semibold">ESC</kbd>
          <span class="hidden md:inline">to return</span>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="relative z-10 max-w-5xl mx-auto px-6 lg:px-12 pt-32 pb-24">
      <Transition
          enter-active-class="transition-all duration-500"
          leave-active-class="transition-all duration-300"
          enter-from-class="opacity-0 translate-y-8"
          leave-to-class="opacity-0"
          mode="out-in"
      >
        <div v-if="note" :key="note.id">
          <!-- Header Section -->
          <div class="mb-12">
            <div class="flex items-start gap-6 mb-6">
              <!-- Type Icon -->
              <div
                  class="relative w-20 h-20 rounded-2xl flex items-center justify-center text-4xl shadow-2xl flex-shrink-0 transition-all duration-300 group hover:scale-105"
                  :class="typeInfo[note.note_type]?.bgColor"
              >
                <div class="absolute inset-0 rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300 blur-xl"
                     :class="`bg-gradient-to-br ${typeInfo[note.note_type]?.gradient}`"></div>
                <span class="relative z-10">{{ typeInfo[note.note_type]?.icon }}</span>
              </div>

              <!-- Meta Info -->
              <div class="flex-1 pt-2">
                <div class="flex items-center gap-3 mb-2">
                  <span
                      class="inline-flex items-center gap-2 text-xs font-bold px-3 py-1.5 rounded-lg border uppercase tracking-widest transition-all duration-300"
                      :class="`${typeInfo[note.note_type]?.bgColor} ${typeInfo[note.note_type]?.textColor} border-${note.note_type === 'harmony' ? 'amber' : note.note_type === 'thought' ? 'purple' : note.note_type === 'phrase' ? 'emerald' : note.note_type === 'rhythm' ? 'fuchsia' : 'blue'}-500/30`"
                  >
                    {{ typeInfo[note.note_type]?.label }}
                  </span>
                </div>
                <p class="text-sm text-slate-500 font-medium">
                  Created on <span class="text-slate-300 font-semibold">{{ formatDate(note.created_at) }}</span>
                </p>
              </div>
            </div>
          </div>

          <!-- Content Card -->
          <article class="relative group">
            <!-- Glow Effect -->
            <div
                class="absolute -inset-1 rounded-3xl opacity-0 group-hover:opacity-30 blur-2xl transition-all duration-500 -z-10"
                :class="`bg-gradient-to-br ${typeInfo[note.note_type]?.gradient}`"
            ></div>

            <div class="relative bg-slate-900 border border-slate-800 rounded-3xl overflow-hidden shadow-2xl">
              <!-- Top Accent Bar -->
              <div
                  class="absolute top-0 left-0 right-0 h-1 opacity-70"
                  :class="`bg-gradient-to-r ${typeInfo[note.note_type]?.gradient}`"
              ></div>

              <div class="p-8 md:p-12" @wheel="handleWheel">
                <!-- Thought Type -->
                <template v-if="note.note_type === 'thought'">
                  <div class="relative">
                    <svg class="absolute -top-4 -left-2 w-12 h-12 text-purple-500/20" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M14.017 21v-7.391c0-5.704 3.731-9.57 8.983-10.609l.995 2.151c-2.432.917-3.995 3.638-3.995 5.849h4v10h-9.983zm-14.017 0v-7.391c0-5.704 3.748-9.57 9-10.609l.996 2.151c-2.433.917-3.996 3.638-3.996 5.849h3.983v10h-9.983z"/>
                    </svg>
                    <blockquote class="relative text-2xl md:text-3xl leading-relaxed text-slate-200 font-serif italic pl-8">
                      {{ note.content }}
                    </blockquote>
                  </div>
                </template>

                <!-- Harmony Type -->
                <template v-else-if="note.note_type === 'harmony'">
                  <div class="bg-slate-950 rounded-2xl border border-slate-800 p-8 overflow-x-auto custom-scrollbar">
                    <pre class="font-mono text-base text-amber-200/90 whitespace-pre-wrap leading-loose">{{ note.content }}</pre>
                  </div>
                </template>

                <!-- Phrase Type -->
                <template v-else-if="note.note_type === 'phrase'">
                  <div class="space-y-8">
                    <!-- Audio Player -->
                    <div class="flex flex-col md:flex-row items-center gap-6 p-8 bg-slate-950/70 rounded-2xl border border-slate-800">
                      <button
                          @click="togglePlay"
                          class="relative w-20 h-20 flex items-center justify-center rounded-full shadow-2xl flex-shrink-0 transition-all duration-300 hover:scale-110 active:scale-95 group/play"
                          :class="`bg-gradient-to-br ${typeInfo[note.note_type]?.gradient}`"
                      >
                        <div class="absolute inset-0 rounded-full blur-xl opacity-50 group-hover/play:opacity-70 transition-opacity"
                             :class="`bg-gradient-to-br ${typeInfo[note.note_type]?.gradient}`"></div>
                        <svg v-if="!isPlaying" class="relative z-10 w-8 h-8 text-slate-950 ml-1" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M8 5v14l11-7z" />
                        </svg>
                        <svg v-else class="relative z-10 w-8 h-8 text-slate-950" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                        </svg>
                      </button>

                      <div class="flex-1 w-full">
                        <div class="h-28 w-full bg-slate-900 rounded-xl overflow-hidden border border-slate-800">
                          <Waveform class="w-full h-full opacity-90" />
                        </div>
                      </div>
                    </div>

                    <!-- Comment -->
                    <div v-if="note.metadata.comment || note.content" class="relative pl-8 border-l-2 border-emerald-500/30">
                      <p class="text-lg md:text-xl text-slate-300 leading-relaxed">
                        {{ note.metadata.comment || note.content }}
                      </p>
                    </div>
                  </div>
                </template>

                <!-- Rhythm Type -->
                <template v-else-if="note.note_type === 'rhythm'">
                  <div class="space-y-8">
                    <!-- Time Signature Display -->
                    <div class="flex items-center gap-8">
                      <div class="relative flex flex-col items-center justify-center w-32 h-32 rounded-2xl bg-slate-950 border-2 border-fuchsia-500/30 shadow-xl">
                        <span class="text-5xl font-serif font-bold bg-gradient-to-br from-fuchsia-400 to-pink-600 bg-clip-text text-transparent">
                          {{ note.metadata.time_signature?.split('/')[0] || '4' }}
                        </span>
                        <div class="w-16 h-0.5 bg-slate-700 my-2"></div>
                        <span class="text-5xl font-serif font-bold text-slate-500">
                          {{ note.metadata.time_signature?.split('/')[1] || '4' }}
                        </span>
                      </div>

                      <div class="h-24 w-px bg-slate-800"></div>

                      <div class="flex-1">
                        <div class="text-xs text-slate-500 uppercase tracking-widest font-bold mb-2">Time Signature</div>
                        <div class="text-2xl font-bold text-slate-300">{{ note.metadata.time_signature || '4/4' }}</div>
                      </div>
                    </div>

                    <!-- Pattern -->
                    <div class="relative p-6 bg-slate-950/70 rounded-2xl border-l-4 border-fuchsia-500">
                      <p class="text-xl md:text-2xl text-slate-200 font-mono leading-relaxed">
                        {{ note.content }}
                      </p>
                    </div>
                  </div>
                </template>

                <!-- Score Type -->
                <template v-else-if="note.note_type === 'score'">
                  <div class="space-y-6">
                    <!-- Score Image -->
                    <div class="relative overflow-hidden rounded-2xl border-2 border-slate-800 bg-slate-100 pattern-grid shadow-inner">
                      <div class="absolute inset-0 pointer-events-none shadow-[inset_0_0_60px_rgba(0,0,0,0.1)] z-10"></div>

                      <div
                          class="overflow-auto max-h-[70vh] flex items-center justify-center p-6 transition-all duration-200"
                          :class="imageScale > 1 ? 'cursor-zoom-out' : 'cursor-zoom-in'"
                      >
                        <img
                            v-if="imagePath"
                            :src="imagePath"
                            alt="Musical Score"
                            :style="{ transform: `scale(${imageScale})` }"
                            class="max-w-full transition-transform duration-200 ease-out origin-center shadow-2xl rounded-lg"
                        />
                        <div v-else class="h-80 flex items-center justify-center">
                          <div class="flex flex-col items-center gap-4">
                            <div class="relative w-16 h-16">
                              <div class="absolute inset-0 rounded-full border-4 border-slate-300 border-t-blue-500 animate-spin"></div>
                            </div>
                            <span class="text-sm text-slate-600 uppercase tracking-widest font-semibold">Loading Score</span>
                          </div>
                        </div>
                      </div>
                    </div>

                    <!-- Score Info -->
                    <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 pt-4">
                      <p v-if="note.content" class="text-lg text-slate-300 font-medium">
                        {{ note.content }}
                      </p>
                      <div class="flex items-center gap-2 text-xs text-slate-500 bg-slate-950 px-4 py-2.5 rounded-xl border border-slate-800 whitespace-nowrap">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7"/>
                        </svg>
                        <span>Hold <kbd class="px-2 py-0.5 mx-1 bg-slate-800 border border-slate-700 rounded text-slate-300 font-mono font-bold text-xs">Ctrl</kbd> + Scroll</span>
                      </div>
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </article>
        </div>

        <!-- Loading State -->
        <div v-else class="flex flex-col items-center justify-center py-32 gap-6">
          <div class="relative w-20 h-20">
            <div class="absolute inset-0 rounded-full border-4 border-transparent border-t-amber-500 animate-spin"></div>
            <div class="absolute inset-2 rounded-full border-4 border-transparent border-t-orange-600 animate-spin animation-delay-350"></div>
          </div>
          <p class="text-xs text-slate-500 tracking-widest uppercase font-semibold">Loading Note</p>
        </div>
      </Transition>
    </main>
  </div>
</template>

<style scoped>
/* Scrollbar Styling */
.custom-scrollbar::-webkit-scrollbar {
  height: 8px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(251, 191, 36, 0.2);
  border-radius: 20px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(251, 191, 36, 0.3);
}

/* Grid Pattern for Score Background */
.pattern-grid {
  background-image:
      linear-gradient(rgba(203, 213, 225, 0.3) 1px, transparent 1px),
      linear-gradient(90deg, rgba(203, 213, 225, 0.3) 1px, transparent 1px);
  background-size: 20px 20px;
}

/* Animations */
@keyframes float {
  0%, 100% {
    transform: translateY(0) translateX(0);
  }
  50% {
    transform: translateY(-40px) translateX(30px);
  }
}

@keyframes float-reverse {
  0%, 100% {
    transform: translateY(0) translateX(0);
  }
  50% {
    transform: translateY(-40px) translateX(-30px);
  }
}

.animate-float {
  animation: float 25s ease-in-out infinite;
}

.animate-float-reverse {
  animation: float-reverse 20s ease-in-out infinite;
}

.animation-delay-350 {
  animation-delay: -0.35s;
}

.bg-gradient-radial {
  background: radial-gradient(circle, var(--tw-gradient-stops));
}
</style>