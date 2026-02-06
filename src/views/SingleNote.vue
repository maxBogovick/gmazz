<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { getAssetPath } from '../api/notes';
import { getSetting, isTauri } from '../api/server';
import Waveform from '../components/common/Waveform.vue';

const route = useRoute();
const router = useRouter();
const store = useNotesStore();
const isPlaying = ref(false);
const audio = ref<HTMLAudioElement | null>(null);
const imagePath = ref<string>('');
const imageScale = ref(1);
const mouseX = ref(0.5);
const mouseY = ref(0.5);
const time = ref(0);
const isReducedMotion = ref(false);
const isLightAmbient = ref(false);
let motionMedia: MediaQueryList | null = null;
let isAnimating = false;

const note = computed(() => store.currentNote);

const typeInfo: Record<string, { label: string; labelRu: string; icon: string; accent: string; gradient: string }> = {
  thought: {
    label: 'Personal Reflection',
    labelRu: 'Личная Заметка',
    icon: '✦',
    accent: '#C4956A',
    gradient: 'linear-gradient(135deg, #FDF8F3 0%, #F9F1E8 100%)'
  },
  harmony: {
    label: 'Harmonic Analysis',
    labelRu: 'Гармонический Анализ',
    icon: '♮',
    accent: '#7B9E87',
    gradient: 'linear-gradient(135deg, #F5F9F6 0%, #EBF4EE 100%)'
  },
  phrase: {
    label: 'Recorded Phrase',
    labelRu: 'Запись Фразы',
    icon: '𝄞',
    accent: '#8B7BA8',
    gradient: 'linear-gradient(135deg, #F8F6FA 0%, #F0ECF5 100%)'
  },
  rhythm: {
    label: 'Rhythmic Pattern',
    labelRu: 'Ритмический Паттерн',
    icon: '◈',
    accent: '#B8856E',
    gradient: 'linear-gradient(135deg, #FBF6F4 0%, #F6EDE8 100%)'
  },
  score: {
    label: 'Original Manuscript',
    labelRu: 'Оригинальная Партитура',
    icon: '𝄚',
    accent: '#6B8FAD',
    gradient: 'linear-gradient(135deg, #F5F8FA 0%, #EAF1F6 100%)'
  },
};

let animationFrame: number;

async function loadResources() {
  if (!note.value) return;

  console.log("current node is = ", note.value);
  // Reset
  if (audio.value) {
    audio.value.pause();
    audio.value = null;
  }
  imagePath.value = '';
  isPlaying.value = false;

  // Load Audio
  if (note.value?.note_type === 'phrase' && note.value.metadata.file_path) {
    const path = await getAssetPath(note.value.metadata.file_path);
    audio.value = new Audio(path);
    audio.value.addEventListener('ended', () => { isPlaying.value = false; });
  } else if (note.value.note_type === 'score' && note.value.metadata.audio_path) {
    const path = await getAssetPath(note.value.metadata.audio_path);
    console.log("audio path = ", path);
    audio.value = new Audio(path);
    audio.value.addEventListener('ended', () => { isPlaying.value = false; });
  }

  // Load Score Image/PDF
  if (note.value?.note_type === 'score' && note.value.metadata.file_path) {
    imagePath.value = await getAssetPath(note.value.metadata.file_path);
  }
}

onMounted(async () => {
  const id = route.params.id as string;
  await store.fetchNote(id);
  await loadResources();

  document.addEventListener('keydown', handleKeydown);
  window.addEventListener('wheel', handleWheel, { passive: false });
  const savedAmbient = isTauri()
    ? await getSetting('gmazz_light_ambient')
    : localStorage.getItem('gmazz_light_ambient');
  isLightAmbient.value = savedAmbient === '1';
  updateMotionPrefs();
  if (motionMedia) {
    motionMedia.addEventListener('change', updateMotionPrefs);
  }
  syncAmbientMotion();
});

watch(note, loadResources);

onUnmounted(() => {
  if (audio.value) {
    audio.value.pause();
    audio.value = null;
  }
  store.clearCurrentNote();
  document.removeEventListener('keydown', handleKeydown);
  window.removeEventListener('wheel', handleWheel);
  window.removeEventListener('mousemove', handleMouseMove);
  if (motionMedia) {
    motionMedia.removeEventListener('change', updateMotionPrefs);
  }
  if (animationFrame) cancelAnimationFrame(animationFrame);
});

function handleMouseMove(e: MouseEvent) {
  requestAnimationFrame(() => {
    mouseX.value = e.clientX / window.innerWidth;
    mouseY.value = e.clientY / window.innerHeight;
  });
}

function updateMotionPrefs() {
  if (!motionMedia) {
    motionMedia = window.matchMedia('(prefers-reduced-motion: reduce)');
  }
  isReducedMotion.value = motionMedia.matches;
  syncAmbientMotion();
}

function syncAmbientMotion() {
  const wantsMotion = !isReducedMotion.value && !isLightAmbient.value;
  if (wantsMotion) {
    window.addEventListener('mousemove', handleMouseMove, { passive: true });
    if (!isAnimating) {
      isAnimating = true;
      const animate = () => {
        time.value += 0.01;
        animationFrame = requestAnimationFrame(animate);
      };
      animate();
    }
  } else {
    window.removeEventListener('mousemove', handleMouseMove);
    if (animationFrame) cancelAnimationFrame(animationFrame);
    isAnimating = false;
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    goBack();
  }
}

function goBack() {
  router.push({ name: 'feed' });
}

function handleEdit() {
  if (!note.value) return;
  router.push({ name: 'edit', params: { id: note.value.id } });
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

const ambientStyle = computed(() => {
  if (!note.value) return {};
  const accent = typeInfo[note.value.note_type]?.accent || '#C4956A';
  const r = parseInt(accent.slice(1, 3), 16);
  const g = parseInt(accent.slice(3, 5), 16);
  const b = parseInt(accent.slice(5, 7), 16);

  if (isReducedMotion.value || isLightAmbient.value) {
    return {
      background: 'linear-gradient(180deg, #FAF7F2 0%, #F6F2EC 50%, #F2EDE6 100%)'
    };
  }

  return {
    background: `
      radial-gradient(ellipse 100% 70% at ${30 + mouseX.value * 25}% ${20 + mouseY.value * 25}%, rgba(${r}, ${g}, ${b}, 0.15) 0%, transparent 65%),
      radial-gradient(ellipse 80% 60% at ${70 - mouseX.value * 20}% ${60 + mouseY.value * 20}%, rgba(${r}, ${g}, ${b}, 0.1) 0%, transparent 65%),
      linear-gradient(180deg, #FAF7F2 0%, #F6F2EC 50%, #F2EDE6 100%)
    `
  };
});
</script>

<template>
  <div class="min-h-screen overflow-x-hidden" :style="ambientStyle">

    <!-- Enhanced Atmospheric Layers -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden" v-if="note">
      <div
          class="absolute -top-1/3 -left-1/4 w-[1000px] h-[1000px] rounded-full opacity-40 will-change-transform blur-[100px]"
          :style="{
          background: `radial-gradient(circle, ${typeInfo[note.note_type]?.accent}40 0%, ${typeInfo[note.note_type]?.accent}20 40%, transparent 70%)`,
          transform: `translate3d(${mouseX * 40}px, ${mouseY * 40}px, 0) scale(${1 + Math.sin(time) * 0.08})`
        }"
      />

      <div
          class="absolute -bottom-1/3 -right-1/4 w-[800px] h-[800px] rounded-full opacity-30 will-change-transform blur-[100px]"
          :style="{
          background: `radial-gradient(circle, ${typeInfo[note.note_type]?.accent}30 0%, ${typeInfo[note.note_type]?.accent}15 40%, transparent 70%)`,
          transform: `translate3d(${-mouseX * 30}px, ${-mouseY * 30}px, 0) scale(${1 + Math.cos(time * 0.7) * 0.08})`
        }"
      />
    </div>

    <!-- Floating Musical Notes -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      <div
          v-for="i in 20"
          :key="i"
          class="absolute will-change-transform font-serif opacity-30"
          :class="i % 3 === 0 ? 'text-2xl' : 'text-xl'"
          :style="{
          left: `${(i * 8) % 100}%`,
          top: `${(i * 11 + 20) % 100}%`,
          color: note ? typeInfo[note.note_type]?.accent : '#C4956A',
          animation: `float-musical-${i % 3} ${8 + i % 5}s ease-in-out infinite`,
          animationDelay: `${i * 0.2}s`,
        }"
      >
        {{ ['𝅝', '𝅗𝅥', '♩', '♪', '♫'][i % 5] }}
      </div>
    </div>

    <!-- Header with Enhanced Glassmorphism -->
    <header
        data-tauri-drag-region
        class="fixed top-0 left-0 right-0 z-50 backdrop-blur-2xl border-b transition-all duration-700"
        :style="{
          backgroundColor: 'rgba(250, 247, 242, 0.9)',
          borderColor: 'rgba(200, 180, 160, 0.3)',
          boxShadow: '0 8px 32px rgba(0,0,0,0.06)'
        }"
    >
      <div class="max-w-6xl mx-auto px-8 lg:px-16 h-20 flex items-center justify-between pointer-events-none">
        <button
            @click="goBack"
            class="flex items-center gap-3 text-sm text-stone-600 hover:text-amber-700 transition-all duration-300 pointer-events-auto group"
        >
          <span class="text-xl group-hover:-translate-x-1 transition-transform duration-300">←</span>
          <span class="font-medium tracking-tight">Вернуться в Архив</span>
        </button>

        <div class="flex items-center gap-5 pointer-events-auto">
          <button
              v-if="note"
              @click="handleEdit"
              class="text-[11px] uppercase tracking-[0.25em] text-amber-600 hover:text-amber-700 transition-colors font-sans font-bold"
          >
            Редактировать
          </button>

          <button
              v-if="note"
              @click="handleDelete"
              class="text-[11px] uppercase tracking-[0.25em] text-stone-500 hover:text-red-600 transition-colors font-sans font-bold"
          >
            Удалить
          </button>

          <div class="hidden md:flex items-center gap-2.5 text-[11px] text-stone-500 font-sans">
            <kbd class="px-3 py-1.5 bg-white/70 border border-stone-300 rounded-lg text-[11px] text-stone-600 tracking-wider shadow-sm font-semibold">ESC</kbd>
            <span class="text-stone-400">для выхода</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Exhibition -->
    <main class="relative z-10 max-w-5xl mx-auto px-8 lg:px-16 pt-36 pb-32">
      <Transition
          enter-active-class="transition-all duration-700 ease-out"
          leave-active-class="transition-all duration-300 ease-in"
          enter-from-class="opacity-0 translate-y-8"
          leave-to-class="opacity-0 -translate-y-8"
          mode="out-in"
      >
        <div v-if="note" :key="note.id" class="space-y-14">

          <!-- Enhanced Exhibit Label -->
          <div class="space-y-10">
            <!-- Catalog Info -->
            <div class="flex items-center gap-5">
              <div class="h-[2px] flex-1 bg-gradient-to-r from-transparent via-stone-300 to-stone-300 rounded-full"></div>
              <div class="flex items-center gap-5 text-[11px] uppercase tracking-[0.3em] font-sans font-bold">
                <span :style="{ color: typeInfo[note.note_type]?.accent }">№ {{ note.id.substring(0, 8) }}</span>
                <span class="text-stone-300">•</span>
                <span class="text-stone-400">{{ typeInfo[note.note_type]?.label }}</span>
              </div>
              <div class="h-[2px] flex-1 bg-gradient-to-l from-transparent via-stone-300 to-stone-300 rounded-full"></div>
            </div>

            <!-- Title Block with Enhanced Typography -->
            <div class="text-center space-y-6">
              <div
                  class="text-7xl animate-float font-serif drop-shadow-lg"
                  :style="{
                  color: typeInfo[note.note_type]?.accent + '80',
                  transform: `scale(${1 + Math.sin(time) * 0.05})`
                }"
              >
                {{ typeInfo[note.note_type]?.icon }}
              </div>
              <h1 class="text-5xl lg:text-6xl font-light tracking-tight text-stone-800 drop-shadow-sm">
                {{ typeInfo[note.note_type]?.labelRu }}
              </h1>
              <p class="text-xl text-stone-500 font-light tracking-wide">
                {{ formatDate(note.created_at) }}
              </p>
            </div>
          </div>

          <!-- Enhanced Exhibition Frame -->
          <article class="relative group">
            <div
                class="relative rounded-[32px] p-2 shadow-2xl transition-all duration-700 overflow-hidden"
                :style="{
                background: 'linear-gradient(135deg, rgba(255,255,255,0.95) 0%, rgba(255,255,255,0.9) 100%)',
                boxShadow: `0 20px 60px ${typeInfo[note.note_type]?.accent}20, 0 8px 20px rgba(0,0,0,0.1)`
              }"
            >
              <!-- Animated border glow -->
              <div
                  class="absolute inset-0 rounded-[32px] opacity-50 group-hover:opacity-100 transition-opacity duration-700"
                  :style="{
                  background: `linear-gradient(135deg, ${typeInfo[note.note_type]?.accent}20 0%, transparent 100%)`
                }"
              ></div>

              <!-- Inner Mat -->
              <div class="relative rounded-[28px] overflow-hidden" :style="{ background: typeInfo[note.note_type]?.gradient }">

                <!-- Enhanced Corner Details -->
                <div class="absolute top-6 left-6 w-8 h-8 border-t-2 border-l-2 rounded-tl-lg transition-all duration-500" :style="{ borderColor: typeInfo[note.note_type]?.accent + '60' }"></div>
                <div class="absolute top-6 right-6 w-8 h-8 border-t-2 border-r-2 rounded-tr-lg transition-all duration-500" :style="{ borderColor: typeInfo[note.note_type]?.accent + '60' }"></div>
                <div class="absolute bottom-6 left-6 w-8 h-8 border-b-2 border-l-2 rounded-bl-lg transition-all duration-500" :style="{ borderColor: typeInfo[note.note_type]?.accent + '60' }"></div>
                <div class="absolute bottom-6 right-6 w-8 h-8 border-b-2 border-r-2 rounded-br-lg transition-all duration-500" :style="{ borderColor: typeInfo[note.note_type]?.accent + '60' }"></div>

                <div class="p-10 lg:p-14 relative z-10">

                  <!-- Thought (Personal Notes) -->
                  <template v-if="note.note_type === 'thought'">
                    <div class="max-w-3xl mx-auto space-y-10">
                      <div class="w-20 h-[2px] rounded-full mx-auto" :style="{ background: typeInfo[note.note_type]?.accent + '60' }"></div>
                      <div class="text-2xl lg:text-3xl leading-relaxed text-stone-700 whitespace-pre-line text-center font-light">
                        "{{ note.content }}"
                      </div>
                      <div class="flex justify-center pt-8">
                        <div class="text-4xl animate-pulse-slow" :style="{ color: typeInfo[note.note_type]?.accent + '70' }">✦</div>
                      </div>
                    </div>
                  </template>

                  <!-- Harmony (Theory) -->
                  <template v-else-if="note.note_type === 'harmony'">
                    <div class="space-y-8">
                      <div class="text-center mb-10">
                        <h3 class="text-[11px] uppercase tracking-[0.35em] font-sans font-bold mb-4" :style="{ color: typeInfo[note.note_type]?.accent }">
                          Гармонический Анализ
                        </h3>
                        <div v-if="note.metadata?.chord_symbol" class="inline-flex items-center gap-3 bg-white/80 px-6 py-3 rounded-xl shadow-md border" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                          <span class="text-[11px] text-stone-500 font-sans font-semibold uppercase tracking-wider">Аккорд:</span>
                          <span class="text-2xl font-mono font-bold" :style="{ color: typeInfo[note.note_type]?.accent }">{{ note.metadata.chord_symbol }}</span>
                        </div>
                      </div>

                      <div class="bg-white/90 border-2 rounded-2xl p-10 overflow-x-auto custom-scrollbar backdrop-blur-sm shadow-inner" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                        <pre class="font-mono text-lg text-stone-700 whitespace-pre-wrap leading-loose">{{ note.content }}</pre>
                      </div>
                    </div>
                  </template>

                  <!-- Phrase (Audio Recording) -->
                  <template v-else-if="note.note_type === 'phrase'">
                    <div class="space-y-12">

                      <!-- Enhanced Playback Console -->
                      <div class="bg-white/90 border-2 rounded-3xl p-10 space-y-8 backdrop-blur-sm shadow-xl" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                        <div class="flex items-center justify-center gap-10">
                          <!-- Enhanced Play Button -->
                          <button
                              @click="togglePlay"
                              class="w-24 h-24 flex items-center justify-center rounded-full text-white hover:scale-110 transition-all duration-500 shadow-2xl relative group overflow-hidden"
                              :style="{
                                background: `linear-gradient(135deg, ${typeInfo[note.note_type]?.accent} 0%, ${typeInfo[note.note_type]?.accent}CC 100%)`,
                                boxShadow: `0 12px 40px ${typeInfo[note.note_type]?.accent}40`
                              }"
                          >
                            <div class="absolute inset-0 rounded-full bg-white/20 opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>

                            <svg v-if="!isPlaying" class="w-10 h-10 ml-1 relative z-10" fill="currentColor" viewBox="0 0 24 24">
                              <path d="M8 5v14l11-7z" />
                            </svg>
                            <svg v-else class="w-10 h-10 relative z-10" fill="currentColor" viewBox="0 0 24 24">
                              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                            </svg>

                            <!-- Enhanced Pulse Effect -->
                            <div v-if="isPlaying" class="absolute inset-0 rounded-full animate-ping opacity-30" :style="{ background: typeInfo[note.note_type]?.accent }"></div>
                            <div v-if="isPlaying" class="absolute inset-0 rounded-full animate-pulse opacity-20" :style="{ background: typeInfo[note.note_type]?.accent }"></div>
                          </button>
                        </div>

                        <!-- Enhanced Waveform Display -->
                        <div class="h-32 w-full bg-white/70 border-2 rounded-2xl relative overflow-hidden shadow-inner" :style="{ borderColor: typeInfo[note.note_type]?.accent + '20' }">
                          <div class="absolute left-1/2 top-0 bottom-0 w-[2px] rounded-full z-20" :style="{ background: typeInfo[note.note_type]?.accent }"></div>
                          <Waveform class="w-full h-full opacity-70" />
                        </div>

                        <div class="text-center">
                          <div class="text-[11px] uppercase tracking-[0.3em] font-sans font-bold" :style="{ color: isPlaying ? typeInfo[note.note_type]?.accent : '#9CA3AF' }">
                            {{ isPlaying ? 'Воспроизведение...' : 'Готов к воспроизведению' }}
                          </div>
                        </div>
                      </div>

                      <!-- Enhanced Annotation -->
                      <div v-if="note.metadata.comment || note.content" class="border-l-[3px] rounded-l pl-10" :style="{ borderColor: typeInfo[note.note_type]?.accent + '50' }">
                        <p class="text-xl lg:text-2xl text-stone-600 italic leading-relaxed font-light">
                          "{{ note.metadata.comment || note.content }}"
                        </p>
                      </div>
                    </div>
                  </template>

                  <!-- Rhythm -->
                  <template v-else-if="note.note_type === 'rhythm'">
                    <div class="max-w-3xl mx-auto space-y-12">

                      <div class="text-center">
                        <h3 class="text-[11px] uppercase tracking-[0.35em] font-sans font-bold mb-10" :style="{ color: typeInfo[note.note_type]?.accent }">
                          Ритмическая Структура
                        </h3>
                      </div>

                      <!-- Enhanced Time Signature Display -->
                      <div class="flex justify-center">
                        <div class="relative">
                          <div class="w-40 h-40 bg-white border-[3px] rounded-2xl flex flex-col items-center justify-center shadow-2xl relative overflow-hidden" :style="{ borderColor: typeInfo[note.note_type]?.accent + '60' }">
                            <div class="absolute inset-0 opacity-10" :style="{ background: typeInfo[note.note_type]?.gradient }"></div>
                            <span class="text-6xl font-light leading-none relative z-10" :style="{ color: typeInfo[note.note_type]?.accent }">
                              {{ note.metadata.time_signature?.split('/')[0] || '4' }}
                            </span>
                            <div class="w-16 h-[2px] rounded-full my-3 relative z-10" :style="{ background: typeInfo[note.note_type]?.accent + '60' }"></div>
                            <span class="text-6xl font-light text-stone-700 leading-none relative z-10">
                              {{ note.metadata.time_signature?.split('/')[1] || '4' }}
                            </span>
                          </div>

                          <!-- Enhanced Corner Accents -->
                          <div class="absolute -top-1 -left-1 w-6 h-6 border-t-[3px] border-l-[3px] rounded-tl-lg" :style="{ borderColor: typeInfo[note.note_type]?.accent }"></div>
                          <div class="absolute -top-1 -right-1 w-6 h-6 border-t-[3px] border-r-[3px] rounded-tr-lg" :style="{ borderColor: typeInfo[note.note_type]?.accent }"></div>
                          <div class="absolute -bottom-1 -left-1 w-6 h-6 border-b-[3px] border-l-[3px] rounded-bl-lg" :style="{ borderColor: typeInfo[note.note_type]?.accent }"></div>
                          <div class="absolute -bottom-1 -right-1 w-6 h-6 border-b-[3px] border-r-[3px] rounded-br-lg" :style="{ borderColor: typeInfo[note.note_type]?.accent }"></div>
                        </div>
                      </div>

                      <!-- Enhanced Pattern -->
                      <div class="bg-white/90 border-2 rounded-2xl p-10 text-center backdrop-blur-sm shadow-inner" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                        <div class="text-[11px] uppercase tracking-[0.3em] mb-5 font-sans font-bold" :style="{ color: typeInfo[note.note_type]?.accent }">
                          Groove Pattern
                        </div>
                        <div class="font-mono text-2xl lg:text-3xl text-stone-700 tracking-wider leading-loose">
                          {{ note.content }}
                        </div>
                      </div>

                      <!-- Mood Tag -->
                      <div v-if="note.metadata?.mood" class="flex justify-center pt-4">
                        <span class="text-sm px-6 py-2 rounded-full bg-white/90 text-stone-600 font-bold shadow-md border-2" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                          {{ note.metadata.mood }}
                        </span>
                      </div>
                    </div>
                  </template>

                  <!-- Score (Sheet Music) -->
                  <template v-else-if="note.note_type === 'score'">
                    <div class="space-y-8">

                      <div class="text-center mb-8">
                        <h3 class="text-[11px] uppercase tracking-[0.35em] font-sans font-bold mb-4" :style="{ color: typeInfo[note.note_type]?.accent }">
                          Оригинальная Партитура
                        </h3>
                        <div v-if="note.metadata?.key" class="inline-flex items-center gap-3 bg-white/80 px-6 py-3 rounded-xl shadow-md border" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                          <span class="text-[11px] text-stone-500 font-sans font-semibold uppercase tracking-wider">Тональность:</span>
                          <span class="text-xl font-bold" :style="{ color: typeInfo[note.note_type]?.accent }">{{ note.metadata.key }}</span>
                        </div>
                      </div>

                      <!-- Audio Player for Score -->
                      <div v-if="audio" class="flex justify-center">
                        <button
                            @click="togglePlay"
                            class="flex items-center gap-4 px-8 py-3 rounded-full text-white transition-all duration-500 shadow-xl hover:scale-105 group"
                            :style="{
                              background: `linear-gradient(135deg, ${typeInfo[note.note_type]?.accent} 0%, ${typeInfo[note.note_type]?.accent}CC 100%)`
                            }"
                        >
                          <span class="text-2xl">{{ isPlaying ? '⏸' : '▶' }}</span>
                          <span class="text-[11px] uppercase tracking-widest font-bold">{{ isPlaying ? 'Пауза' : 'Слушать запись' }}</span>
                        </button>
                      </div>

                      <!-- Enhanced Score Viewer -->
                      <div class="relative bg-white/80 p-4 rounded-2xl border-2 backdrop-blur-sm shadow-xl" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                        <div class="bg-white overflow-auto max-h-[70vh] flex items-center justify-center p-8 rounded-xl"
                             :class="imageScale > 1 ? 'cursor-zoom-out' : 'cursor-zoom-in'">
                          <img
                              v-if="imagePath"
                              :src="imagePath"
                              alt="Musical Score"
                              :style="{ transform: `scale(${imageScale})` }"
                              class="max-w-full shadow-2xl transition-transform duration-300 ease-out origin-center rounded-lg"
                          />
                          <div v-else class="h-96 flex items-center justify-center text-stone-500 font-sans">
                            <div class="flex flex-col items-center gap-4">
                              <div class="text-5xl animate-pulse-slow" :style="{ color: typeInfo[note.note_type]?.accent }">𝄞</div>
                              <div class="text-sm tracking-wider">Загрузка партитуры...</div>
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Score Info -->
                      <div class="flex justify-between items-center text-[11px] text-stone-500 font-sans px-2">
                        <span class="text-stone-700 font-semibold">{{ note.content }}</span>
                        <span class="uppercase tracking-wider" :style="{ color: typeInfo[note.note_type]?.accent }">Ctrl + Scroll для масштабирования</span>
                      </div>
                    </div>
                  </template>
                </div>

                <!-- Enhanced Signature Stamp -->
                <div class="px-10 lg:px-14 pb-10">
                  <div class="pt-8 border-t-2 flex justify-between items-center text-[11px] uppercase tracking-[0.3em] font-sans transition-colors duration-500" :style="{ borderColor: typeInfo[note.note_type]?.accent + '30' }">
                    <span class="text-stone-600 font-bold">Архив Gmazz</span>
                    <span class="font-bold" :style="{ color: typeInfo[note.note_type]?.accent }">{{ new Date(note.created_at).getFullYear() }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Decorative elements on hover -->
            <div class="absolute -inset-4 rounded-[40px] opacity-0 group-hover:opacity-100 transition-all duration-1000 pointer-events-none blur-2xl" :style="{ background: `radial-gradient(circle at center, ${typeInfo[note.note_type]?.accent}15 0%, transparent 70%)` }"></div>
          </article>

          <!-- Enhanced Metadata Section -->
          <div v-if="note.metadata && Object.keys(note.metadata).length > 0" class="mt-16">
            <div class="max-w-3xl mx-auto">
              <div class="flex items-center gap-5 mb-8">
                <div class="h-[2px] flex-1 bg-gradient-to-r from-transparent to-stone-300 rounded-full"></div>
                <h3 class="text-[11px] uppercase tracking-[0.35em] text-stone-400 font-sans font-bold">Метаданные</h3>
                <div class="h-[2px] flex-1 bg-gradient-to-l from-transparent to-stone-300 rounded-full"></div>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div
                    v-for="(value, key) in note.metadata"
                    :key="key"
                    v-show="key !== 'file_path' && key !== 'comment' && value"
                    class="bg-white/80 backdrop-blur-sm border-2 rounded-2xl p-5 transition-all duration-500 hover:shadow-lg hover:scale-105"
                    :style="{ borderColor: typeInfo[note.note_type]?.accent + '20' }"
                >
                  <div class="text-[11px] uppercase tracking-wider text-stone-400 mb-2 font-sans font-semibold">{{ key.replace(/_/g, ' ') }}</div>
                  <div class="text-base text-stone-700 font-medium">{{ value }}</div>
                </div>
              </div>
            </div>
          </div>

        </div>

        <!-- Enhanced Loading State -->
        <div v-else class="min-h-[70vh] flex flex-col items-center justify-center space-y-8">
          <div class="relative w-24 h-24">
            <div class="absolute inset-0 border-[3px] border-amber-200 rounded-full animate-ping opacity-25"></div>
            <div class="absolute inset-0 border-[3px] border-amber-300 border-t-amber-700 rounded-full animate-spin"></div>
            <div class="absolute inset-0 flex items-center justify-center">
              <div class="text-4xl text-amber-600 animate-pulse-slow font-serif">𝄞</div>
            </div>
          </div>
          <p class="text-sm uppercase tracking-[0.35em] text-stone-400 font-sans font-bold">
            Извлечение из архива...
          </p>
        </div>
      </Transition>
    </main>

    <!-- Enhanced Footer -->
    <footer class="relative px-8 lg:px-16 py-16 border-t border-stone-200/80 bg-gradient-to-b from-transparent to-stone-50/70 overflow-hidden mt-20">
      <div class="absolute inset-0 opacity-30">
        <div class="absolute top-0 left-1/4 w-64 h-64 bg-amber-200/20 rounded-full blur-[100px]" />
        <div class="absolute bottom-0 right-1/4 w-64 h-64 bg-orange-200/20 rounded-full blur-[100px]" />
      </div>

      <div class="max-w-5xl mx-auto relative">
        <div class="flex flex-col md:flex-row items-center justify-between gap-8">
          <div class="flex items-center gap-5 group cursor-pointer">
            <span class="text-4xl text-amber-400 group-hover:scale-125 group-hover:rotate-12 transition-all duration-700 font-serif drop-shadow-lg" :style="{ transform: `scale(${1 + Math.sin(time * 0.5) * 0.08})` }">𝄞</span>
            <div>
              <span class="text-stone-700 font-bold text-xl group-hover:text-amber-700 transition-colors duration-500 tracking-tight">Gmazz</span>
              <span class="text-[11px] text-stone-400 block tracking-[0.25em] uppercase mt-1 font-semibold">архив 1974—{{ new Date().getFullYear() }}</span>
            </div>
          </div>
          <p class="text-base text-stone-500 italic text-center md:text-right max-w-md leading-relaxed relative">
            <span class="absolute -top-5 -left-5 text-4xl text-amber-300/40 font-serif">"</span>
            Музыка — это то, что происходит между нотами
            <span class="absolute -bottom-5 -right-5 text-4xl text-amber-300/40 font-serif">"</span>
          </p>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
@keyframes float-musical-0 {
  0%, 100% {
    transform: translate3d(0, 0, 0) rotate(0deg);
    opacity: 0.2;
  }
  25% {
    transform: translate3d(15px, -25px, 0) rotate(90deg);
    opacity: 0.5;
  }
  50% {
    transform: translate3d(-8px, -15px, 0) rotate(180deg);
    opacity: 0.3;
  }
  75% {
    transform: translate3d(8px, -30px, 0) rotate(270deg);
    opacity: 0.4;
  }
}

@keyframes float-musical-1 {
  0%, 100% {
    transform: translate3d(0, 0, 0) rotate(0deg);
    opacity: 0.3;
  }
  33% {
    transform: translate3d(-12px, -20px, 0) rotate(120deg);
    opacity: 0.6;
  }
  66% {
    transform: translate3d(12px, -28px, 0) rotate(240deg);
    opacity: 0.35;
  }
}

@keyframes float-musical-2 {
  0%, 100% {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.25;
  }
  50% {
    transform: translate3d(10px, -35px, 0) scale(1.3);
    opacity: 0.55;
  }
}

@keyframes pulse-slow {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.05);
  }
}

@keyframes float {
  0%, 100% {
    transform: translateY(0px) scale(1);
  }
  50% {
    transform: translateY(-15px) scale(1.05);
  }
}

.animate-pulse-slow {
  animation: pulse-slow 3s ease-in-out infinite;
}

.animate-float {
  animation: float 4s ease-in-out infinite;
}

.custom-scrollbar::-webkit-scrollbar {
  height: 10px;
  width: 10px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(245, 241, 232, 0.5);
  border-radius: 5px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(196, 149, 106, 0.4);
  border-radius: 5px;
  transition: background 0.3s ease;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(196, 149, 106, 0.6);
}

* {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.will-change-transform {
  will-change: transform;
}

/* Smooth transitions for all interactive elements */
button {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Smooth scroll behavior */
html {
  scroll-behavior: smooth;
}

/* Enhanced focus states for accessibility */
button:focus-visible,
kbd:focus-visible {
  outline: 2px solid rgba(196, 149, 106, 0.6);
  outline-offset: 2px;
}

/* Prevent text selection on interactive elements */
button,
kbd {
  user-select: none;
  -webkit-user-select: none;
}

/* Optimize rendering performance */
.group > div {
  backface-visibility: hidden;
  transform: translateZ(0);
}
</style>
