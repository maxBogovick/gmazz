<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import type { Note, NoteType } from '../types';

const router = useRouter();
const store = useNotesStore();
const mouseX = ref(0.5);
const mouseY = ref(0.5);

const typeConfig: Record<NoteType, { name: string; icon: string; accent: string; bg: string }> = {
  thought: {
    name: 'Мысль',
    icon: '✦',
    accent: '#C4956A',
    bg: 'linear-gradient(135deg, #FDF8F3 0%, #F9F1E8 100%)'
  },
  harmony: {
    name: 'Гармония',
    icon: '♮',
    accent: '#7B9E87',
    bg: 'linear-gradient(135deg, #F5F9F6 0%, #EBF4EE 100%)'
  },
  phrase: {
    name: 'Фраза',
    icon: '𝄞',
    accent: '#8B7BA8',
    bg: 'linear-gradient(135deg, #F8F6FA 0%, #F0ECF5 100%)'
  },
  rhythm: {
    name: 'Ритм',
    icon: '◈',
    accent: '#B8856E',
    bg: 'linear-gradient(135deg, #FBF6F4 0%, #F6EDE8 100%)'
  },
  score: {
    name: 'Партитура',
    icon: '𝄚',
    accent: '#6B8FAD',
    bg: 'linear-gradient(135deg, #F5F8FA 0%, #EAF1F6 100%)'
  },
};

onMounted(async () => {
  await store.fetchNotes();
  window.addEventListener('mousemove', handleMouseMove);
});

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove);
});

function handleMouseMove(e: MouseEvent) {
  mouseX.value = e.clientX / window.innerWidth;
  mouseY.value = e.clientY / window.innerHeight;
}

function openNote(note: Note) {
  router.push({ name: 'note', params: { id: note.id } });
}

function openCreate() {
  router.push({ name: 'create' });
}

async function openRandomNote() {
  const note = await store.fetchRandomNote();
  if (note) router.push({ name: 'note', params: { id: note.id } });
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'short'
  });
}

function formatYear(dateStr: string): string {
  return new Date(dateStr).getFullYear().toString();
}

function truncate(text: string, len: number): string {
  return text.length > len ? text.slice(0, len) + '…' : text;
}

const ambientStyle = computed(() => ({
  background: `
    radial-gradient(ellipse 80% 50% at ${30 + mouseX.value * 20}% ${20 + mouseY.value * 20}%, rgba(255, 220, 180, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse 60% 40% at ${70 - mouseX.value * 15}% ${60 + mouseY.value * 15}%, rgba(200, 180, 160, 0.1) 0%, transparent 50%),
    linear-gradient(180deg, #F8F4F0 0%, #F5F0EA 50%, #F2EBE4 100%)
  `
}));
</script>

<template>
  <div class="min-h-screen" :style="ambientStyle">

    <!-- Atmospheric Layers -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      <!-- Warm light from top-left -->
      <div
        class="absolute -top-1/4 -left-1/4 w-[800px] h-[800px] rounded-full opacity-40"
        style="background: radial-gradient(circle, rgba(255, 235, 210, 0.6) 0%, transparent 70%);"
        :style="{ transform: `translate(${mouseX * 30}px, ${mouseY * 30}px)` }"
      ></div>

      <!-- Subtle mist layers -->
      <div
        class="absolute top-1/3 right-0 w-full h-64 opacity-30"
        style="background: linear-gradient(90deg, transparent 0%, rgba(245, 240, 235, 0.8) 50%, transparent 100%);"
      ></div>
    </div>

    <!-- Floating Dust Particles -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden opacity-40">
      <div
        v-for="i in 20"
        :key="i"
        class="absolute w-1 h-1 rounded-full bg-amber-200/50"
        :style="{
          left: `${(i * 5) % 100}%`,
          top: `${(i * 7 + 10) % 100}%`,
          animation: `float ${8 + i % 4}s ease-in-out infinite`,
          animationDelay: `${i * 0.3}s`
        }"
      ></div>
    </div>

    <!-- Header -->
    <header class="fixed top-0 left-0 right-0 z-50 backdrop-blur-md bg-[#F8F4F0]/70">
      <div class="max-w-7xl mx-auto px-8 lg:px-16 h-20 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-amber-100 to-orange-100 flex items-center justify-center text-amber-700 text-lg shadow-sm">
            𝄞
          </div>
          <div>
            <span class="text-xl font-medium text-stone-700">Gmazz</span>
            <span class="text-[10px] text-stone-400 block -mt-0.5">архив маэстро</span>
          </div>
        </div>

        <div class="flex items-center gap-6">
          <button
            @click="openRandomNote"
            class="text-sm text-stone-500 hover:text-stone-700 transition-colors flex items-center gap-2"
          >
            <span class="text-amber-600">✦</span>
            случайная
          </button>
          <button
            @click="openCreate"
            class="text-sm text-stone-600 bg-white/80 hover:bg-white px-5 py-2.5 rounded-xl shadow-sm hover:shadow transition-all"
          >
            + новая запись
          </button>
        </div>
      </div>
    </header>

    <!-- Hero Section -->
    <section class="relative pt-32 pb-16 px-8 lg:px-16">
      <div class="max-w-7xl mx-auto">
        <div class="max-w-2xl">
          <!-- Decorative element -->
          <div class="flex items-center gap-4 mb-6">
            <div class="text-3xl text-amber-300">❧</div>
            <div class="h-px flex-1 bg-gradient-to-r from-amber-200 to-transparent"></div>
          </div>

          <h1 class="text-5xl lg:text-6xl font-light text-stone-700 leading-tight mb-6">
            Пятьдесят один
            <span class="block text-stone-400">год в джазе</span>
          </h1>

          <p class="text-lg text-stone-500 leading-relaxed max-w-lg">
            Мысли, гармонии, мелодические фразы и партитуры —
            <span class="text-stone-600">живой архив музыкальных идей</span>
          </p>

          <!-- Quick Stats -->
          <div class="flex items-center gap-8 mt-10">
            <div class="text-center">
              <span class="block text-3xl font-light text-stone-600">{{ store.notes.length || '—' }}</span>
              <span class="text-xs text-stone-400">записей</span>
            </div>
            <div class="w-px h-8 bg-stone-200"></div>
            <div class="text-center">
              <span class="block text-3xl font-light text-stone-600">5</span>
              <span class="text-xs text-stone-400">типов</span>
            </div>
            <div class="w-px h-8 bg-stone-200"></div>
            <div class="text-center">
              <span class="block text-3xl font-light text-stone-600">1974</span>
              <span class="text-xs text-stone-400">начало</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Notes Collection -->
    <section class="relative px-8 lg:px-16 pb-24">
      <div class="max-w-7xl mx-auto">

        <!-- Section Header -->
        <div class="flex items-center justify-between mb-10">
          <h2 class="text-sm uppercase tracking-widest text-stone-400">Коллекция работ</h2>
          <div class="h-px flex-1 mx-8 bg-gradient-to-r from-stone-200 via-stone-200 to-transparent"></div>
        </div>

        <!-- Bento-style Grid -->
        <div v-if="store.filteredNotes.length > 0" class="grid grid-cols-12 gap-5">
          <article
            v-for="(note, index) in store.filteredNotes"
            :key="note.id"
            @click="openNote(note)"
            class="group cursor-pointer"
            :class="[
              // Varying sizes for visual interest
              index % 7 === 0 ? 'col-span-12 md:col-span-8' :
              index % 7 === 1 ? 'col-span-12 md:col-span-4' :
              index % 7 === 2 ? 'col-span-12 md:col-span-4' :
              index % 7 === 3 ? 'col-span-12 md:col-span-4' :
              index % 7 === 4 ? 'col-span-12 md:col-span-4' :
              index % 7 === 5 ? 'col-span-12 md:col-span-6' :
              'col-span-12 md:col-span-6'
            ]"
          >
            <div
              class="relative h-full rounded-2xl p-6 transition-all duration-500 group-hover:shadow-xl group-hover:-translate-y-1 overflow-hidden"
              :style="{ background: typeConfig[note.note_type].bg }"
              :class="index % 7 === 0 ? 'min-h-[280px]' : 'min-h-[220px]'"
            >
              <!-- Ambient glow on hover -->
              <div
                class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
                :style="{
                  background: `radial-gradient(circle at 30% 30%, ${typeConfig[note.note_type].accent}15 0%, transparent 60%)`
                }"
              ></div>

              <!-- Decorative corner element -->
              <div
                class="absolute top-4 right-4 text-4xl opacity-10 group-hover:opacity-20 transition-opacity"
                :style="{ color: typeConfig[note.note_type].accent }"
              >
                {{ typeConfig[note.note_type].icon }}
              </div>

              <!-- Card Content -->
              <div class="relative z-10 h-full flex flex-col">
                <!-- Header -->
                <div class="flex items-center gap-3 mb-4">
                  <span
                    class="w-8 h-8 rounded-lg flex items-center justify-center text-sm"
                    :style="{
                      backgroundColor: typeConfig[note.note_type].accent + '20',
                      color: typeConfig[note.note_type].accent
                    }"
                  >
                    {{ typeConfig[note.note_type].icon }}
                  </span>
                  <span class="text-xs font-medium" :style="{ color: typeConfig[note.note_type].accent }">
                    {{ typeConfig[note.note_type].name }}
                  </span>
                  <span class="text-xs text-stone-400 ml-auto">
                    {{ formatDate(note.created_at) }}
                  </span>
                </div>

                <!-- Main Content -->
                <div class="flex-1">
                  <!-- Thought: Large quote style -->
                  <div v-if="note.note_type === 'thought'" class="h-full flex flex-col">
                    <p class="text-lg lg:text-xl text-stone-600 leading-relaxed font-light flex-1">
                      «{{ truncate(note.content, index % 7 === 0 ? 280 : 150) }}»
                    </p>
                  </div>

                  <!-- Harmony: Code block style -->
                  <div v-else-if="note.note_type === 'harmony'" class="h-full flex flex-col">
                    <div class="flex-1 bg-white/50 rounded-xl p-4">
                      <pre class="font-mono text-sm text-stone-600 whitespace-pre-wrap leading-relaxed">{{ truncate(note.content, index % 7 === 0 ? 250 : 120) }}</pre>
                    </div>
                    <div v-if="note.metadata.chord_symbol" class="mt-3 flex items-center gap-2">
                      <span class="text-xs text-stone-400">аккорд:</span>
                      <span class="text-sm font-mono text-stone-600">{{ note.metadata.chord_symbol }}</span>
                    </div>
                  </div>

                  <!-- Phrase: Audio-like style -->
                  <div v-else-if="note.note_type === 'phrase'" class="h-full flex flex-col">
                    <div class="flex items-center gap-4 mb-4">
                      <div
                        class="w-12 h-12 rounded-full flex items-center justify-center text-white shadow-lg group-hover:scale-110 transition-transform"
                        :style="{ backgroundColor: typeConfig[note.note_type].accent }"
                      >
                        ▶
                      </div>
                      <!-- Waveform visualization -->
                      <div class="flex-1 flex items-center gap-0.5 h-8">
                        <div
                          v-for="j in 24"
                          :key="j"
                          class="flex-1 rounded-full transition-all"
                          :style="{
                            height: `${20 + Math.sin(j * 0.5) * 60}%`,
                            backgroundColor: typeConfig[note.note_type].accent + '40'
                          }"
                        ></div>
                      </div>
                    </div>
                    <p class="text-stone-500 text-sm flex-1">{{ truncate(note.content, 100) }}</p>
                  </div>

                  <!-- Rhythm: Time signature focus -->
                  <div v-else-if="note.note_type === 'rhythm'" class="h-full flex flex-col">
                    <div class="flex items-start gap-6">
                      <div
                        class="text-4xl font-light"
                        :style="{ color: typeConfig[note.note_type].accent }"
                      >
                        {{ note.metadata.time_signature || '4/4' }}
                      </div>
                      <div class="flex-1">
                        <p class="text-stone-500 leading-relaxed">{{ truncate(note.content, 120) }}</p>
                        <div v-if="note.metadata.mood" class="mt-3">
                          <span class="text-xs px-2 py-1 rounded-full bg-white/60 text-stone-500">
                            {{ note.metadata.mood }}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- Score: Document preview style -->
                  <div v-else-if="note.note_type === 'score'" class="h-full flex flex-col">
                    <div class="flex-1 bg-white/40 rounded-xl p-4 border border-stone-200/50">
                      <!-- Staff lines -->
                      <div class="relative h-16 mb-3">
                        <div v-for="j in 5" :key="j" class="absolute left-0 right-0 h-px bg-stone-300/50" :style="{ top: `${j * 20}%` }"></div>
                        <div class="absolute left-4 top-1/2 -translate-y-1/2 text-2xl text-stone-400">𝄞</div>
                      </div>
                      <p class="text-stone-500 text-sm">{{ truncate(note.content, 100) }}</p>
                    </div>
                    <div v-if="note.metadata.key" class="mt-3 text-xs text-stone-400">
                      Тональность: <span class="text-stone-600">{{ note.metadata.key }}</span>
                    </div>
                  </div>
                </div>

                <!-- Footer -->
                <div class="flex items-center justify-between mt-4 pt-4 border-t border-stone-200/50">
                  <span class="text-xs text-stone-400 font-mono">#{{ note.id.substring(0, 6) }}</span>
                  <span
                    class="text-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1"
                    :style="{ color: typeConfig[note.note_type].accent }"
                  >
                    открыть <span class="text-base">→</span>
                  </span>
                </div>
              </div>
            </div>
          </article>
        </div>

        <!-- Empty State -->
        <div v-else-if="!store.loading" class="text-center py-32">
          <div class="inline-block p-8 rounded-3xl bg-white/50 backdrop-blur-sm">
            <div class="text-6xl text-amber-200 mb-6">𝄞</div>
            <h3 class="text-xl text-stone-600 mb-3">Архив пуст</h3>
            <p class="text-stone-400 mb-6 max-w-sm">
              Создайте первую запись, чтобы начать формировать коллекцию
            </p>
            <button
              @click="openCreate"
              class="text-sm text-stone-600 bg-white hover:bg-stone-50 px-6 py-3 rounded-xl shadow-sm hover:shadow transition-all"
            >
              + создать запись
            </button>
          </div>
        </div>

        <!-- Loading -->
        <div v-if="store.loading" class="text-center py-32">
          <div class="inline-block">
            <div class="w-10 h-10 border-2 border-amber-200 border-t-amber-500 rounded-full animate-spin mb-4"></div>
            <p class="text-sm text-stone-400">загрузка...</p>
          </div>
        </div>

        <!-- Load More -->
        <div v-if="store.hasMore && store.filteredNotes.length > 0" class="text-center mt-12">
          <button
            @click="store.loadMore()"
            :disabled="store.loading"
            class="text-sm text-stone-500 border border-stone-300 px-8 py-3 rounded-xl hover:bg-white/50 hover:border-stone-400 transition-all disabled:opacity-50"
          >
            показать ещё
          </button>
        </div>
      </div>
    </section>

    <!-- Footer -->
    <footer class="relative px-8 lg:px-16 py-12 border-t border-stone-200/50">
      <div class="max-w-7xl mx-auto">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-2xl text-amber-300">𝄞</span>
            <div>
              <span class="text-stone-600">Gmazz</span>
              <span class="text-xs text-stone-400 block">архив 1974—{{ new Date().getFullYear() }}</span>
            </div>
          </div>
          <p class="text-sm text-stone-400 italic">
            «Музыка — это то, что происходит между нотами»
          </p>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
@keyframes float {
  0%, 100% {
    transform: translateY(0) translateX(0);
    opacity: 0.3;
  }
  25% {
    transform: translateY(-20px) translateX(10px);
    opacity: 0.6;
  }
  50% {
    transform: translateY(-10px) translateX(-5px);
    opacity: 0.4;
  }
  75% {
    transform: translateY(-30px) translateX(5px);
    opacity: 0.5;
  }
}

/* Smooth rendering */
* {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
