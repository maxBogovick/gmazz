<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import type { Note, NoteType } from '../types';

const router = useRouter();
const store = useNotesStore();

const dialAngle = ref(0);
const selectedIndex = ref(0);
const isPlaying = ref(false);
const signalStrength = ref(0);
const volumeLevel = ref(0.7);

const typeInfo: Record<NoteType, { freq: string; band: string; icon: string }> = {
  thought: { freq: '88.1', band: 'FM', icon: '✎' },
  harmony: { freq: '91.5', band: 'FM', icon: '♯' },
  phrase: { freq: '95.3', band: 'FM', icon: '♪' },
  rhythm: { freq: '101.7', band: 'FM', icon: '◎' },
  score: { freq: '107.9', band: 'FM', icon: '𝄞' },
};

const currentNote = computed(() => store.filteredNotes[selectedIndex.value] || null);
const currentFreq = computed(() => {
  if (!currentNote.value) return '88.0';
  const base = 88 + (selectedIndex.value * 0.2);
  return base.toFixed(1);
});

watch(selectedIndex, () => {
  // Animate signal search
  signalStrength.value = 0;
  const interval = setInterval(() => {
    signalStrength.value += 0.1;
    if (signalStrength.value >= 1) {
      clearInterval(interval);
      signalStrength.value = 1;
    }
  }, 50);
});

onMounted(async () => {
  await store.fetchNotes();
  signalStrength.value = 1;
});

function handleDialRotate(e: WheelEvent) {
  e.preventDefault();
  const delta = e.deltaY > 0 ? 1 : -1;
  const newIndex = Math.max(0, Math.min(store.filteredNotes.length - 1, selectedIndex.value + delta));
  if (newIndex !== selectedIndex.value) {
    selectedIndex.value = newIndex;
    dialAngle.value += delta * 15;
  }
}

function selectStation(index: number) {
  selectedIndex.value = index;
  dialAngle.value = index * 15;
}

function openNote(note: Note) {
  router.push({ name: 'note', params: { id: note.id } });
}

function openCreate() {
  router.push({ name: 'create' });
}

async function openRandomNote() {
  const note = await store.fetchRandomNote();
  if (note) {
    router.push({ name: 'note', params: { id: note.id } });
  }
}

function formatDate(dateStr: string): string {
  const d = new Date(dateStr);
  const months = ['янв', 'фев', 'мар', 'апр', 'май', 'июн', 'июл', 'авг', 'сен', 'окт', 'ноя', 'дек'];
  return `${d.getDate()} ${months[d.getMonth()]} ${d.getFullYear()}`;
}

function truncate(text: string, length: number): string {
  if (text.length <= length) return text;
  return text.slice(0, length) + '...';
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-b from-[#F5E6D3] via-[#E8D4BC] to-[#DCC5A8] overflow-hidden">

    <!-- Wooden Texture Overlay -->
    <div class="fixed inset-0 pointer-events-none opacity-10"
         style="background-image: url('data:image/svg+xml,%3Csvg width=&quot;100&quot; height=&quot;100&quot; viewBox=&quot;0 0 100 100&quot; xmlns=&quot;http://www.w3.org/2000/svg&quot;%3E%3Cpath d=&quot;M0 0h100v2H0zM0 20h100v1H0zM0 35h100v2H0zM0 55h100v1H0zM0 75h100v2H0zM0 90h100v1H0z&quot; fill=&quot;%238B4513&quot; fill-opacity=&quot;0.3&quot;/%3E%3C/svg%3E');">
    </div>

    <!-- Header -->
    <header class="relative z-10 px-8 py-6">
      <div class="max-w-6xl mx-auto flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-full bg-gradient-to-br from-[#C9A227] to-[#8B6914] flex items-center justify-center shadow-lg">
            <span class="text-white text-xl">𝄞</span>
          </div>
          <div>
            <h1 class="text-2xl font-light text-[#5C4A36] tracking-wide">Gmazz Radio</h1>
            <p class="text-xs text-[#8B7355] tracking-widest uppercase">Архив Маэстро • Est. 1974</p>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <button
            @click="openRandomNote"
            class="px-5 py-2.5 border-2 border-[#8B7355] text-[#5C4A36] hover:bg-[#8B7355] hover:text-white rounded-full text-sm transition-all"
          >
            Случайная волна
          </button>
          <button
            @click="openCreate"
            class="px-5 py-2.5 bg-[#C9A227] text-white hover:bg-[#B8860B] rounded-full text-sm shadow-lg transition-all"
          >
            + Новая запись
          </button>
        </div>
      </div>
    </header>

    <!-- Main Radio Console -->
    <main class="relative z-10 max-w-6xl mx-auto px-8 py-8">

      <!-- The Radio Cabinet -->
      <div class="bg-gradient-to-b from-[#8B5A2B] via-[#6B4423] to-[#5C3A1D] rounded-3xl p-8 shadow-2xl border-4 border-[#4A2C17]">

        <!-- Top Decorative Grille -->
        <div class="h-4 bg-[#4A2C17] rounded-t-xl mb-6 relative overflow-hidden">
          <div class="absolute inset-0 flex">
            <div v-for="i in 40" :key="i" class="flex-1 border-r border-[#3D2415]"></div>
          </div>
        </div>

        <!-- Display Section -->
        <div class="grid grid-cols-3 gap-6 mb-8">

          <!-- Left: VU Meter Style Display -->
          <div class="bg-[#F5E6D3] rounded-2xl p-6 shadow-inner">
            <div class="text-center mb-4">
              <div class="text-[10px] uppercase tracking-[0.3em] text-[#8B7355] mb-2">Сигнал</div>
              <div class="flex justify-center gap-1">
                <div
                  v-for="i in 10"
                  :key="i"
                  class="w-3 h-8 rounded-sm transition-all duration-150"
                  :class="i <= signalStrength * 10 ? (i > 7 ? 'bg-[#C9A227]' : 'bg-[#8B7355]') : 'bg-[#D4CAB5]'"
                ></div>
              </div>
            </div>

            <div class="text-center">
              <div class="text-[10px] uppercase tracking-[0.3em] text-[#8B7355] mb-2">Год записи</div>
              <div class="text-3xl font-light text-[#5C4A36]">
                {{ currentNote ? new Date(currentNote.created_at).getFullYear() : '—' }}
              </div>
            </div>
          </div>

          <!-- Center: Main Frequency Display -->
          <div
            class="bg-gradient-to-b from-[#1a1510] to-[#0d0a08] rounded-2xl p-6 relative overflow-hidden"
            @wheel="handleDialRotate"
          >
            <!-- Glow effect -->
            <div class="absolute inset-0 bg-[#C9A227] opacity-5 rounded-2xl"></div>

            <!-- Frequency Scale -->
            <div class="relative h-20 mb-4 overflow-hidden">
              <div class="absolute inset-0 flex items-center justify-center">
                <!-- Scale marks -->
                <div class="relative w-full h-full">
                  <div
                    v-for="i in 20"
                    :key="i"
                    class="absolute top-1/2 -translate-y-1/2 text-[#C9A227]/40 text-[10px] font-mono"
                    :style="{ left: `${i * 5}%` }"
                  >
                    |
                  </div>
                </div>

                <!-- Center indicator -->
                <div class="absolute top-0 left-1/2 -translate-x-1/2 w-0.5 h-full bg-[#C9A227]"></div>

                <!-- Frequency numbers -->
                <div
                  class="absolute inset-0 flex items-center transition-transform duration-300"
                  :style="{ transform: `translateX(${-selectedIndex * 30}px)` }"
                >
                  <div
                    v-for="(note, index) in store.filteredNotes"
                    :key="note.id"
                    class="flex-shrink-0 w-[30px] text-center"
                  >
                    <span
                      class="text-[10px] font-mono transition-all"
                      :class="index === selectedIndex ? 'text-[#C9A227] text-lg' : 'text-[#666]'"
                    >
                      {{ (88 + index * 0.2).toFixed(1) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Current Frequency -->
            <div class="text-center">
              <div class="text-5xl font-light text-[#C9A227] tracking-wider font-mono mb-2">
                {{ currentFreq }}
                <span class="text-2xl text-[#C9A227]/60">FM</span>
              </div>
              <div class="text-[10px] uppercase tracking-[0.3em] text-[#666]">
                {{ currentNote ? typeInfo[currentNote.note_type].icon : '◎' }}
                {{ store.filteredNotes.length }} станций
              </div>
            </div>

            <!-- Tuning hint -->
            <div class="absolute bottom-2 left-0 right-0 text-center">
              <span class="text-[8px] text-[#555] uppercase tracking-widest">прокрутка для настройки</span>
            </div>
          </div>

          <!-- Right: Station Info -->
          <div class="bg-[#F5E6D3] rounded-2xl p-6 shadow-inner">
            <div class="text-center">
              <div class="text-[10px] uppercase tracking-[0.3em] text-[#8B7355] mb-3">Категория</div>

              <div v-if="currentNote" class="space-y-3">
                <div class="text-4xl">{{ typeInfo[currentNote.note_type].icon }}</div>
                <div class="text-lg text-[#5C4A36] font-light">
                  {{ currentNote.note_type === 'thought' ? 'Мысль' :
                     currentNote.note_type === 'harmony' ? 'Гармония' :
                     currentNote.note_type === 'phrase' ? 'Фраза' :
                     currentNote.note_type === 'rhythm' ? 'Ритм' : 'Партитура' }}
                </div>
              </div>

              <div v-else class="text-[#8B7355]">
                Поиск...
              </div>
            </div>

            <div class="mt-6 text-center">
              <div class="text-[10px] uppercase tracking-[0.3em] text-[#8B7355] mb-2">Всего записей</div>
              <div class="text-2xl font-light text-[#C9A227]">{{ store.filteredNotes.length }}</div>
            </div>
          </div>
        </div>

        <!-- Speaker Grille with Content -->
        <div class="bg-[#4A2C17] rounded-2xl p-1">
          <div class="bg-gradient-to-b from-[#F5E6D3] to-[#E8D4BC] rounded-xl p-8 relative">

            <!-- Grille pattern overlay -->
            <div class="absolute inset-0 opacity-5 rounded-xl"
                 style="background-image: radial-gradient(circle, #000 1px, transparent 1px); background-size: 8px 8px;">
            </div>

            <!-- Current Note Content -->
            <div v-if="currentNote" class="relative z-10">
              <div class="flex items-start justify-between mb-6">
                <div>
                  <div class="text-sm text-[#8B7355] mb-1">{{ formatDate(currentNote.created_at) }}</div>
                  <div class="text-[10px] uppercase tracking-widest text-[#A89F8B]">
                    № {{ currentNote.id.substring(0, 8) }}
                  </div>
                </div>
                <button
                  @click="openNote(currentNote)"
                  class="px-6 py-3 bg-[#C9A227] text-white hover:bg-[#B8860B] rounded-full text-sm shadow-lg transition-all flex items-center gap-2"
                >
                  <span>Открыть полностью</span>
                  <span>→</span>
                </button>
              </div>

              <!-- Content Display -->
              <div class="min-h-[200px]">
                <!-- Thought -->
                <div v-if="currentNote.note_type === 'thought'" class="max-w-3xl">
                  <p class="text-2xl lg:text-3xl font-light text-[#3D2F1E] leading-relaxed italic">
                    "{{ currentNote.content }}"
                  </p>
                </div>

                <!-- Harmony -->
                <div v-else-if="currentNote.note_type === 'harmony'">
                  <pre class="font-mono text-xl text-[#704214] whitespace-pre-wrap leading-loose bg-white/50 p-6 rounded-lg">{{ currentNote.content }}</pre>
                </div>

                <!-- Phrase -->
                <div v-else-if="currentNote.note_type === 'phrase'" class="text-center py-8">
                  <div class="w-32 h-32 mx-auto rounded-full border-4 border-[#C9A227] flex items-center justify-center mb-6 hover:bg-[#C9A227]/10 cursor-pointer transition-colors">
                    <span class="text-5xl text-[#C9A227]">▶</span>
                  </div>
                  <p class="text-xl text-[#5C4A36] italic">{{ currentNote.content }}</p>
                </div>

                <!-- Rhythm -->
                <div v-else-if="currentNote.note_type === 'rhythm'" class="text-center py-8">
                  <div class="text-7xl font-light text-[#C9A227] mb-6 font-mono">
                    {{ currentNote.metadata.time_signature || '4/4' }}
                  </div>
                  <p class="text-xl text-[#5C4A36]">{{ currentNote.content }}</p>
                </div>

                <!-- Score -->
                <div v-else-if="currentNote.note_type === 'score'" class="text-center py-8">
                  <div class="w-48 h-32 mx-auto border-4 border-dashed border-[#C9A227]/50 rounded-lg flex items-center justify-center mb-6 bg-white/30">
                    <span class="text-6xl text-[#C9A227]/50">𝄞</span>
                  </div>
                  <p class="text-xl text-[#5C4A36]">{{ currentNote.content }}</p>
                </div>
              </div>
            </div>

            <!-- Empty State -->
            <div v-else-if="!store.loading && store.filteredNotes.length === 0" class="text-center py-16">
              <div class="text-6xl text-[#C9A227]/30 mb-6">📻</div>
              <h3 class="text-2xl text-[#5C4A36] mb-4">Эфир пуст</h3>
              <p class="text-[#8B7355] mb-6">Создайте первую запись для вещания</p>
              <button
                @click="openCreate"
                class="px-8 py-3 bg-[#C9A227] text-white hover:bg-[#B8860B] rounded-full shadow-lg transition-all"
              >
                Начать запись
              </button>
            </div>

            <!-- Loading -->
            <div v-else-if="store.loading" class="text-center py-16">
              <div class="text-6xl text-[#C9A227] animate-pulse mb-4">◎</div>
              <p class="text-[#8B7355] uppercase tracking-widest text-sm">Поиск сигнала...</p>
            </div>
          </div>
        </div>

        <!-- Bottom Control Knobs -->
        <div class="flex justify-center gap-16 mt-8">
          <!-- Volume Knob -->
          <div class="text-center">
            <div class="w-16 h-16 rounded-full bg-gradient-to-b from-[#C9A227] to-[#8B6914] shadow-lg flex items-center justify-center cursor-pointer hover:scale-105 transition-transform">
              <div class="w-1 h-6 bg-[#5C4A36] rounded-full" style="transform: rotate(-30deg);"></div>
            </div>
            <div class="text-[10px] uppercase tracking-widest text-[#D4CAB5] mt-3">Громкость</div>
          </div>

          <!-- Tuning Knob -->
          <div class="text-center">
            <div
              class="w-20 h-20 rounded-full bg-gradient-to-b from-[#C9A227] to-[#8B6914] shadow-lg flex items-center justify-center cursor-pointer hover:scale-105 transition-transform"
              :style="{ transform: `rotate(${dialAngle}deg)` }"
            >
              <div class="w-1 h-8 bg-[#5C4A36] rounded-full"></div>
            </div>
            <div class="text-[10px] uppercase tracking-widest text-[#D4CAB5] mt-3">Настройка</div>
          </div>

          <!-- Tone Knob -->
          <div class="text-center">
            <div class="w-16 h-16 rounded-full bg-gradient-to-b from-[#C9A227] to-[#8B6914] shadow-lg flex items-center justify-center cursor-pointer hover:scale-105 transition-transform">
              <div class="w-1 h-6 bg-[#5C4A36] rounded-full" style="transform: rotate(45deg);"></div>
            </div>
            <div class="text-[10px] uppercase tracking-widest text-[#D4CAB5] mt-3">Тембр</div>
          </div>
        </div>
      </div>

      <!-- Station List (like radio presets) -->
      <div class="mt-8">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg text-[#5C4A36] font-light">Все станции</h3>
          <span class="text-sm text-[#8B7355]">{{ store.filteredNotes.length }} записей</span>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
          <button
            v-for="(note, index) in store.filteredNotes"
            :key="note.id"
            @click="selectStation(index)"
            class="p-4 rounded-xl text-left transition-all"
            :class="index === selectedIndex
              ? 'bg-[#C9A227] text-white shadow-lg scale-105'
              : 'bg-white/60 hover:bg-white text-[#5C4A36] hover:shadow-md'"
          >
            <div class="flex items-center gap-2 mb-2">
              <span class="text-lg">{{ typeInfo[note.note_type].icon }}</span>
              <span class="font-mono text-sm">{{ (88 + index * 0.2).toFixed(1) }}</span>
            </div>
            <div class="text-[10px] uppercase tracking-wider opacity-70 truncate">
              {{ truncate(note.content, 25) }}
            </div>
          </button>
        </div>

        <!-- Load More -->
        <div v-if="store.hasMore" class="text-center mt-8">
          <button
            @click="store.loadMore()"
            :disabled="store.loading"
            class="px-8 py-3 border-2 border-[#8B7355] text-[#5C4A36] hover:bg-[#8B7355] hover:text-white rounded-full transition-all disabled:opacity-50"
          >
            {{ store.loading ? 'Поиск...' : 'Найти ещё станции' }}
          </button>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="relative z-10 py-8 text-center">
      <p class="text-sm text-[#8B7355]">
        Gmazz Radio Archive © 1974—{{ new Date().getFullYear() }}
      </p>
      <p class="text-xs text-[#A89F8B] mt-1">
        «Музыка — это то, что происходит между нотами»
      </p>
    </footer>
  </div>
</template>

<style scoped>
</style>
