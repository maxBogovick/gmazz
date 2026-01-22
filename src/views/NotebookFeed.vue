<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import type { Note, NoteType } from '../types';
import ThoughtCard from '../components/notes/ThoughtCard.vue';
import HarmonyCard from '../components/notes/HarmonyCard.vue';
import PhraseCard from '../components/notes/PhraseCard.vue';
import RhythmCard from '../components/notes/RhythmCard.vue';
import ScoreCard from '../components/notes/ScoreCard.vue';

const router = useRouter();
const store = useNotesStore();
const showFilter = ref(false);
const visibleNotes = ref<string[]>([]);
const scrollY = ref(0);
const mouseX = ref(0);
const mouseY = ref(0);

const noteTypes: { type: NoteType; label: string; icon: string }[] = [
  { type: 'thought', label: 'Thoughts', icon: '✨' },
  { type: 'harmony', label: 'Harmonies', icon: '🎵' },
  { type: 'phrase', label: 'Phrases', icon: '📝' },
  { type: 'rhythm', label: 'Rhythms', icon: '⚡' },
  { type: 'score', label: 'Scores', icon: '🎼' },
];

const headerScrolled = computed(() => scrollY.value > 50);

const cursorStyle = computed(() => ({
  left: `${mouseX.value}px`,
  top: `${mouseY.value}px`,
}));

onMounted(async () => {
  await store.fetchNotes();
  animateNotes();

  window.addEventListener('scroll', handleScroll);
  window.addEventListener('mousemove', handleMouseMove);
});

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll);
  window.removeEventListener('mousemove', handleMouseMove);
});

function handleScroll() {
  scrollY.value = window.scrollY;
}

function handleMouseMove(e: MouseEvent) {
  mouseX.value = e.clientX;
  mouseY.value = e.clientY;
}

function animateNotes() {
  visibleNotes.value = [];
  store.filteredNotes.forEach((note, index) => {
    setTimeout(() => {
      visibleNotes.value.push(note.id);
    }, index * 50);
  });
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

function toggleFilter() {
  showFilter.value = !showFilter.value;
}

function setTypeFilter(type: NoteType | undefined) {
  store.setFilter(type);
  visibleNotes.value = [];
  showFilter.value = false;
  setTimeout(animateNotes, 100);
}

function getCardComponent(type: NoteType) {
  const components: Record<NoteType, any> = {
    thought: ThoughtCard,
    harmony: HarmonyCard,
    phrase: PhraseCard,
    rhythm: RhythmCard,
    score: ScoreCard,
  };
  return components[type];
}

function getTypeConfig(type: NoteType) {
  return noteTypes.find(t => t.type === type);
}
</script>

<template>
  <div class="min-h-screen relative overflow-x-hidden bg-[#12141a] text-gray-200 font-sans">
    <!-- Animated Background Orbs -->
    <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden">
      <div class="absolute rounded-full blur-[100px] opacity-15 w-[600px] h-[600px] -top-[250px] -left-[200px] bg-[radial-gradient(circle,rgba(251,191,36,1)_0%,transparent_70%)] animate-[float_25s_ease-in-out_infinite]"></div>
      <div class="absolute rounded-full blur-[100px] opacity-15 w-[500px] h-[500px] top-[40%] -right-[150px] bg-[radial-gradient(circle,rgba(245,158,11,1)_0%,transparent_70%)] animate-[float_20s_ease-in-out_infinite_reverse]"></div>
      <div class="absolute rounded-full blur-[100px] opacity-15 w-[550px] h-[550px] -bottom-[200px] left-[35%] bg-[radial-gradient(circle,rgba(217,119,6,1)_0%,transparent_70%)] animate-[float_22s_ease-in-out_infinite]"></div>
    </div>

    <!-- Cursor Glow Effect -->
    <div
        class="hidden lg:block fixed w-[400px] h-[400px] rounded-full pointer-events-none -translate-x-1/2 -translate-y-1/2 z-10 transition-opacity duration-300 bg-[radial-gradient(circle,rgba(251,191,36,0.08)_0%,transparent_70%)]"
        :style="cursorStyle"
    ></div>

    <!-- Header -->
    <header
        class="fixed top-0 left-0 right-0 z-50 transition-all duration-500 border-b border-transparent"
        :class="{ 'bg-[#12141a]/85 backdrop-blur-2xl border-white/10 shadow-xl': headerScrolled }"
    >
      <div class="max-w-[1400px] mx-auto px-6 h-20 flex items-center justify-between gap-6 md:px-8">
        <!-- Brand -->
        <div class="flex items-center gap-4 shrink-0 group cursor-pointer">
          <div class="relative">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-amber-400 to-orange-600 flex items-center justify-center text-[#12141a] shadow-[0_0_20px_rgba(251,191,36,0.3)] transition-transform duration-300 group-hover:scale-105 group-hover:rotate-[5deg]">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
              </svg>
            </div>
          </div>
          <div class="flex flex-col gap-0.5">
            <h1 class="text-xl font-bold text-white tracking-tight leading-tight">Digital Notebook</h1>
            <p class="text-xs text-gray-400 font-medium hidden sm:block">Your musical sanctuary</p>
          </div>
        </div>

        <!-- Desktop Navigation -->
        <nav class="hidden lg:block flex-1 max-w-[700px]">
          <div class="flex items-center gap-2 bg-[#1a1d24] border border-white/10 rounded-2xl p-1.5">
            <button
                @click="setTypeFilter(undefined)"
                class="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all duration-300 relative overflow-hidden group/nav"
                :class="!store.filter.note_type ? 'text-[#12141a] font-semibold' : 'text-gray-400 hover:text-white hover:bg-white/5'"
            >
              <div v-if="!store.filter.note_type" class="absolute inset-0 bg-gradient-to-r from-amber-400 to-orange-500"></div>
              <span class="relative z-10">📚</span>
              <span class="relative z-10 xl:inline hidden">All</span>
            </button>
            <button
                v-for="{ type, label, icon } in noteTypes"
                :key="type"
                @click="setTypeFilter(type)"
                class="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all duration-300 relative overflow-hidden"
                :class="store.filter.note_type === type ? 'text-[#12141a] font-semibold' : 'text-gray-400 hover:text-white hover:bg-white/5'"
            >
              <div v-if="store.filter.note_type === type" class="absolute inset-0 bg-gradient-to-r from-amber-400 to-orange-500"></div>
              <span class="relative z-10">{{ icon }}</span>
              <span class="relative z-10 xl:inline hidden">{{ label }}</span>
            </button>
          </div>
        </nav>

        <!-- Actions -->
        <div class="flex items-center gap-4 shrink-0">
          <button
              @click="toggleFilter"
              class="lg:hidden flex items-center justify-center w-10 h-10 rounded-xl bg-[#1a1d24] border border-white/10 text-gray-400 hover:text-white hover:border-white/20 transition-colors"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
            </svg>
          </button>

          <button
              v-if="store.filteredNotes.length > 0"
              @click="openRandomNote"
              class="hidden sm:flex items-center gap-2 px-4 py-2.5 rounded-xl bg-[#1a1d24] border border-white/10 text-gray-400 text-sm font-medium hover:text-white hover:border-amber-400/50 hover:bg-[#20242c] transition-all group"
          >
            <svg class="w-5 h-5 transition-transform duration-700 group-hover:rotate-180" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span class="hidden sm:inline">Random</span>
          </button>

          <button
              @click="openCreate"
              class="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-amber-400 to-orange-600 text-[#12141a] text-sm font-bold shadow-[0_0_15px_rgba(251,191,36,0.4)] hover:shadow-[0_0_25px_rgba(251,191,36,0.6)] hover:-translate-y-0.5 active:translate-y-0 transition-all duration-300"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            <span class="hidden sm:inline">New Entry</span>
          </button>
        </div>
      </div>
    </header>

    <!-- Mobile Filter Dropdown -->
    <Transition name="filter-dropdown">
      <div v-if="showFilter" class="fixed top-[80px] left-0 right-0 z-40 bg-[#12141a]/95 backdrop-blur-2xl border-b border-white/10 p-6 shadow-2xl">
        <div class="flex flex-wrap gap-2">
          <button
              @click="setTypeFilter(undefined)"
              class="flex items-center gap-2 px-4 py-3 rounded-xl text-sm font-medium border transition-all"
              :class="!store.filter.note_type ? 'bg-gradient-to-r from-amber-400 to-orange-500 border-transparent text-[#12141a] font-bold' : 'bg-[#1a1d24] border-white/10 text-gray-400 hover:border-white/20 hover:text-white'"
          >
            <span>📚</span>
            <span>All Notes</span>
          </button>
          <button
              v-for="{ type, label, icon } in noteTypes"
              :key="type"
              @click="setTypeFilter(type)"
              class="flex items-center gap-2 px-4 py-3 rounded-xl text-sm font-medium border transition-all"
              :class="store.filter.note_type === type ? 'bg-gradient-to-r from-amber-400 to-orange-500 border-transparent text-[#12141a] font-bold' : 'bg-[#1a1d24] border-white/10 text-gray-400 hover:border-white/20 hover:text-white'"
          >
            <span>{{ icon }}</span>
            <span>{{ label }}</span>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Main Content -->
    <main class="relative z-20 max-w-[1400px] mx-auto pt-36 px-6 pb-24 md:px-8">
      <!-- Stats -->
      <div v-if="store.filteredNotes.length > 0" class="inline-flex items-center gap-3 px-5 py-3 bg-[#1a1d24] border border-white/10 rounded-2xl mb-8">
        <span class="text-2xl font-extrabold text-transparent bg-clip-text bg-gradient-to-r from-amber-400 to-orange-500">
          {{ store.filteredNotes.length }}
        </span>
        <span class="text-xs text-gray-500 uppercase tracking-widest font-semibold">
          {{ store.filter.note_type ? getTypeConfig(store.filter.note_type)?.label : 'Total Notes' }}
        </span>
      </div>

      <!-- Notes Grid -->
      <div v-if="store.filteredNotes.length > 0" class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fill,minmax(340px,1fr))] gap-8">
        <TransitionGroup name="note">
          <article
              v-for="note in store.filteredNotes"
              :key="note.id"
              v-show="visibleNotes.includes(note.id)"
              class="relative"
          >
            <div
                class="group relative cursor-pointer rounded-2xl transition-all duration-500"
                @click="openNote(note)"
            >
              <!-- Glow Effect -->
              <div class="absolute -inset-[3px] bg-gradient-to-r from-amber-500 to-orange-600 rounded-2xl opacity-0 blur-lg transition-opacity duration-500 group-hover:opacity-50 -z-10"></div>

              <!-- Card Content -->
              <div class="bg-[#1a1d24] border border-white/10 rounded-2xl p-6 relative overflow-hidden transition-all duration-500 group-hover:border-white/20 group-hover:-translate-y-1.5 group-hover:shadow-2xl">
                <!-- Top Accent Bar -->
                <div class="absolute top-0 left-0 right-0 h-[3px] bg-gradient-to-r from-amber-400 to-orange-600 opacity-0 transition-opacity duration-500 group-hover:opacity-100"></div>

                <!-- Header -->
                <div class="flex items-center justify-between mb-6">
                  <div class="flex items-center gap-3">
                    <div class="w-11 h-11 flex items-center justify-center text-xl bg-[#12141a] border border-white/10 rounded-lg transition-all duration-300 group-hover:bg-gradient-to-br group-hover:from-amber-400 group-hover:to-orange-600 group-hover:border-transparent group-hover:scale-110 group-hover:rotate-[8deg] group-hover:shadow-lg">
                      {{ getTypeConfig(note.note_type)?.icon }}
                    </div>
                    <span class="text-xs font-semibold text-gray-500 px-2.5 py-1 bg-[#232730] rounded uppercase tracking-widest">
                      {{ getTypeConfig(note.note_type)?.label }}
                    </span>
                  </div>

                  <svg class="w-5 h-5 text-gray-600 transition-all duration-300 group-hover:text-amber-400 group-hover:translate-x-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </div>

                <!-- Body -->
                <div class="pointer-events-none">
                  <component :is="getCardComponent(note.note_type)" :note="note" />
                </div>
              </div>
            </div>
          </article>
        </TransitionGroup>
      </div>

      <!-- Load More Button -->
      <div v-if="store.hasMore && store.filteredNotes.length > 0" class="flex justify-center mt-12">
        <button
            @click="store.loadMore()"
            :disabled="store.loading"
            class="group relative inline-flex items-center gap-3 px-8 py-3 rounded-xl bg-[#1a1d24] border border-white/10 text-gray-400 font-medium transition-all duration-300 hover:text-white hover:border-amber-400/50 hover:bg-[#20242c] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span v-if="!store.loading">Load More</span>
          <span v-else>Loading...</span>
          
          <svg v-if="!store.loading" class="w-5 h-5 transition-transform duration-300 group-hover:translate-y-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
          <svg v-else class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </button>
      </div>

      <!-- Empty State -->
      <div v-else-if="!store.loading" class="flex flex-col items-center justify-center py-24 px-4 text-center">
        <div class="relative mb-12">
          <!-- Pulse Rings -->
          <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[180px] h-[180px] rounded-full border border-white/5 animate-[pulse_4.5s_ease-in-out_infinite]"></div>
          <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[140px] h-[140px] rounded-full border border-white/5 animate-[pulse_4.5s_ease-in-out_infinite] [animation-delay:1.5s]"></div>
          <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[100px] h-[100px] rounded-full border border-white/5 animate-[pulse_4.5s_ease-in-out_infinite] [animation-delay:3s]"></div>

          <!-- Icon -->
          <div class="relative w-20 h-20 rounded-2xl bg-gradient-to-br from-amber-400 to-orange-600 flex items-center justify-center text-[#12141a] shadow-[0_0_30px_rgba(251,191,36,0.4)] z-10">
            <svg class="w-9 h-9" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          </div>
        </div>

        <h2 class="text-3xl font-bold text-white mb-4 tracking-tight">Your notebook awaits</h2>
        <p class="max-w-[420px] text-sm leading-relaxed text-gray-400 mb-8">
          Begin your creative journey. Capture melodies, harmonies, and musical inspirations.
        </p>

        <button
            @click="openCreate"
            class="inline-flex items-center gap-3 px-8 py-4 rounded-xl bg-gradient-to-r from-amber-400 to-orange-600 text-[#12141a] text-sm font-bold shadow-lg transition-all duration-300 hover:-translate-y-1 hover:shadow-amber-500/40 active:translate-y-0"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          <span>Create First Entry</span>
        </button>
      </div>

      <!-- Loading State -->
      <div v-else class="flex flex-col items-center justify-center py-24 gap-6">
        <div class="relative w-20 h-20">
          <div class="absolute inset-0 rounded-full border-[3px] border-transparent border-t-amber-400 animate-spin"></div>
          <div class="absolute inset-2.5 rounded-full border-[3px] border-transparent border-t-amber-600 animate-spin [animation-delay:-0.35s]"></div>
        </div>
        <p class="text-xs text-gray-500 tracking-widest uppercase">Loading your notes...</p>
      </div>
    </main>
  </div>
</template>

<style>
/* Vue Transitions */
.filter-dropdown-enter-active,
.filter-dropdown-leave-active {
  transition: all 0.3s ease;
}

.filter-dropdown-enter-from,
.filter-dropdown-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

.note-enter-active {
  transition: all 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.note-enter-from {
  opacity: 0;
  transform: translateY(50px) scale(0.9);
}

.note-leave-active {
  transition: all 0.3s ease;
}

.note-leave-to {
  opacity: 0;
  transform: scale(0.92);
}

/* Custom Keyframe for Float */
@keyframes float {
  0% { transform: translateY(0) translateX(0); }
  50% { transform: translateY(-40px) translateX(30px); }
  100% { transform: translateY(0) translateX(0); }
}
</style>