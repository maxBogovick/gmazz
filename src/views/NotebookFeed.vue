<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import type { Note, NoteType } from '../types';

const router = useRouter();
const store = useNotesStore();
const isLoaded = ref(false);
const hoveredNote = ref<string | null>(null);

const sections: { type: NoteType | 'all'; name: string; instrument: string; clef: string }[] = [
  { type: 'all', name: 'Tutti', instrument: 'Полная партитура', clef: '𝄞' },
  { type: 'score', name: 'Партитура', instrument: 'Full Score', clef: '𝄞' },
  { type: 'harmony', name: 'Гармония', instrument: 'Harmonic Analysis', clef: '𝄢' },
  { type: 'phrase', name: 'Фразы', instrument: 'Melodic Lines', clef: '𝄞' },
  { type: 'rhythm', name: 'Ритм', instrument: 'Percussion', clef: '𝄥' },
  { type: 'thought', name: 'Заметки', instrument: 'Composer Notes', clef: '✎' },
];

const activeSection = ref<NoteType | 'all'>('all');

const filteredNotes = computed(() => {
  if (activeSection.value === 'all') return store.filteredNotes;
  return store.filteredNotes.filter(n => n.note_type === activeSection.value);
});

// Group notes by year for "movements"
const notesByYear = computed(() => {
  const grouped: Record<number, Note[]> = {};
  filteredNotes.value.forEach(note => {
    const year = new Date(note.created_at).getFullYear();
    if (!grouped[year]) grouped[year] = [];
    grouped[year].push(note);
  });
  return Object.entries(grouped)
    .map(([year, notes]) => ({ year: parseInt(year), notes }))
    .sort((a, b) => b.year - a.year);
});

onMounted(async () => {
  await store.fetchNotes();
  setTimeout(() => isLoaded.value = true, 100);
});

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

function setSection(type: NoteType | 'all') {
  activeSection.value = type;
}

function getRehearsalMark(index: number): string {
  const marks = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
  if (index < marks.length) return marks[index];
  return `${marks[Math.floor(index / marks.length) - 1]}${marks[index % marks.length]}`;
}

function getMeasureNumber(yearIndex: number, noteIndex: number): number {
  let measure = 1;
  for (let i = 0; i < yearIndex; i++) {
    measure += notesByYear.value[i].notes.length;
  }
  return measure + noteIndex;
}

function formatDate(dateStr: string): string {
  const d = new Date(dateStr);
  return `${d.getDate()}.${d.getMonth() + 1}.${d.getFullYear()}`;
}

function truncate(text: string, len: number): string {
  return text.length > len ? text.slice(0, len) + '...' : text;
}

const typeSymbols: Record<NoteType, { symbol: string; dynamic: string }> = {
  thought: { symbol: '♩', dynamic: 'p' },
  harmony: { symbol: '𝄢', dynamic: 'mf' },
  phrase: { symbol: '♫', dynamic: 'f' },
  rhythm: { symbol: '𝅘𝅥𝅮', dynamic: 'ff' },
  score: { symbol: '𝄞', dynamic: 'fff' },
};
</script>

<template>
  <div class="min-h-screen bg-[#FDF8F0]">

    <!-- Manuscript Paper Lines (CSS background) -->
    <div class="fixed inset-0 pointer-events-none opacity-40"
         style="background-image:
           repeating-linear-gradient(transparent, transparent 23px, #C4B59D 23px, #C4B59D 24px),
           repeating-linear-gradient(transparent, transparent 119px, #A69580 119px, #A69580 120px);
           background-size: 100% 120px;">
    </div>

    <!-- Title Block (like score header) -->
    <header class="relative z-10 pt-12 pb-8 px-8 border-b-2 border-[#8B7355]">
      <div class="max-w-5xl mx-auto text-center">

        <!-- Composer info -->
        <div class="mb-6">
          <div class="text-[11px] uppercase tracking-[0.5em] text-[#8B7355] mb-2">
            Собрание сочинений
          </div>
          <h1 class="text-5xl lg:text-7xl font-light text-[#2C1810] tracking-wide" style="font-family: 'Crimson Text', 'Times New Roman', serif;">
            GMAZZ
          </h1>
          <div class="text-lg text-[#5C4A36] mt-2 italic" style="font-family: 'Crimson Text', serif;">
            Рабочие записи аранжировщика
          </div>
        </div>

        <!-- Score metadata -->
        <div class="flex justify-center gap-12 text-sm text-[#8B7355]">
          <div class="text-center">
            <div class="text-2xl text-[#2C1810] font-light">{{ store.notes.length }}</div>
            <div class="text-[10px] uppercase tracking-widest">записей</div>
          </div>
          <div class="text-center">
            <div class="text-2xl text-[#2C1810] font-light">1974—{{ new Date().getFullYear() }}</div>
            <div class="text-[10px] uppercase tracking-widest">период</div>
          </div>
          <div class="text-center">
            <div class="text-2xl text-[#2C1810] font-light">51</div>
            <div class="text-[10px] uppercase tracking-widest">год работы</div>
          </div>
        </div>

        <!-- Action buttons (like conductor marks) -->
        <div class="flex justify-center gap-4 mt-8">
          <button
            @click="openRandomNote"
            class="px-6 py-2 border border-[#8B7355] text-[#5C4A36] hover:bg-[#8B7355] hover:text-white text-sm transition-all"
            style="font-family: 'Crimson Text', serif;"
          >
            Случайная страница
          </button>
          <button
            @click="openCreate"
            class="px-6 py-2 bg-[#2C1810] text-[#FDF8F0] hover:bg-[#1a0f0a] text-sm transition-all"
            style="font-family: 'Crimson Text', serif;"
          >
            + Новая запись
          </button>
        </div>
      </div>
    </header>

    <!-- Instrument/Section Labels (like orchestral score) -->
    <nav class="sticky top-0 z-20 bg-[#FDF8F0]/95 backdrop-blur-sm border-b border-[#C4B59D] py-3">
      <div class="max-w-5xl mx-auto px-8">
        <div class="flex gap-1 overflow-x-auto scrollbar-hide">
          <button
            v-for="section in sections"
            :key="section.type"
            @click="setSection(section.type)"
            class="flex-shrink-0 px-4 py-2 text-sm transition-all border-l-2"
            :class="activeSection === section.type
              ? 'border-[#2C1810] bg-[#2C1810] text-[#FDF8F0]'
              : 'border-[#C4B59D] hover:border-[#8B7355] text-[#5C4A36] hover:bg-[#F5EDE0]'"
          >
            <span class="text-lg mr-2">{{ section.clef }}</span>
            <span style="font-family: 'Crimson Text', serif;">{{ section.name }}</span>
          </button>
        </div>
      </div>
    </nav>

    <!-- The Score -->
    <main class="relative z-10 max-w-5xl mx-auto px-8 py-12">

      <!-- Movements (grouped by year) -->
      <div v-if="notesByYear.length > 0" class="space-y-16">
        <section
          v-for="(group, groupIndex) in notesByYear"
          :key="group.year"
          class="relative"
        >
          <!-- Movement header (Rehearsal mark) -->
          <div class="flex items-center gap-4 mb-8">
            <!-- Rehearsal mark box -->
            <div class="w-12 h-12 border-2 border-[#2C1810] flex items-center justify-center text-xl font-bold text-[#2C1810]">
              {{ getRehearsalMark(groupIndex) }}
            </div>

            <div class="flex-1">
              <div class="flex items-baseline gap-4">
                <h2 class="text-3xl font-light text-[#2C1810]" style="font-family: 'Crimson Text', serif;">
                  {{ group.year }}
                </h2>
                <span class="text-sm text-[#8B7355] italic">
                  {{ group.notes.length }} {{ group.notes.length === 1 ? 'запись' : 'записей' }}
                </span>
              </div>
              <!-- Double bar line -->
              <div class="h-0.5 bg-[#2C1810] mt-2"></div>
              <div class="h-0.5 bg-[#2C1810] mt-0.5"></div>
            </div>
          </div>

          <!-- Measures (notes as musical phrases) -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <article
              v-for="(note, noteIndex) in group.notes"
              :key="note.id"
              @click="openNote(note)"
              @mouseenter="hoveredNote = note.id"
              @mouseleave="hoveredNote = null"
              class="relative cursor-pointer group"
              :class="isLoaded ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'"
              :style="{ transitionDelay: `${noteIndex * 50}ms`, transition: 'all 0.5s ease-out' }"
            >
              <!-- The measure/bar -->
              <div class="bg-white/70 backdrop-blur-sm border border-[#C4B59D] p-6 hover:shadow-lg transition-all relative overflow-hidden">

                <!-- Bar line left -->
                <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-[#2C1810]"></div>

                <!-- Measure number -->
                <div class="absolute top-2 left-3 text-[10px] text-[#8B7355] font-mono">
                  m.{{ getMeasureNumber(groupIndex, noteIndex) }}
                </div>

                <!-- Clef and type indicator -->
                <div class="flex items-start gap-4 mb-4">
                  <div class="text-3xl text-[#8B7355]">
                    {{ typeSymbols[note.note_type].symbol }}
                  </div>
                  <div class="flex-1">
                    <div class="text-[10px] uppercase tracking-widest text-[#8B7355] mb-1">
                      {{ note.note_type === 'thought' ? 'Заметка' :
                         note.note_type === 'harmony' ? 'Гармония' :
                         note.note_type === 'phrase' ? 'Фраза' :
                         note.note_type === 'rhythm' ? 'Ритм' : 'Партитура' }}
                    </div>
                    <div class="text-xs text-[#A69580]">
                      {{ formatDate(note.created_at) }}
                    </div>
                  </div>
                  <!-- Dynamic marking -->
                  <div class="text-lg italic text-[#8B7355]" style="font-family: 'Crimson Text', serif;">
                    {{ typeSymbols[note.note_type].dynamic }}
                  </div>
                </div>

                <!-- Content (the "music") -->
                <div class="relative">
                  <!-- Staff lines behind content -->
                  <div class="absolute inset-0 opacity-20">
                    <div v-for="i in 5" :key="i" class="h-px bg-[#8B7355] mb-3 first:mt-2"></div>
                  </div>

                  <!-- Thought -->
                  <p
                    v-if="note.note_type === 'thought'"
                    class="relative text-[#2C1810] leading-relaxed italic text-lg"
                    style="font-family: 'Crimson Text', serif;"
                  >
                    "{{ truncate(note.content, 120) }}"
                  </p>

                  <!-- Harmony -->
                  <pre
                    v-else-if="note.note_type === 'harmony'"
                    class="relative font-mono text-sm text-[#5C4A36] whitespace-pre-wrap leading-loose"
                  >{{ truncate(note.content, 100) }}</pre>

                  <!-- Phrase -->
                  <div v-else-if="note.note_type === 'phrase'" class="relative flex items-center gap-4">
                    <div class="w-12 h-12 rounded-full border-2 border-[#8B7355] flex items-center justify-center group-hover:bg-[#8B7355] group-hover:text-white transition-colors">
                      <span class="text-xl">▶</span>
                    </div>
                    <p class="text-[#5C4A36] italic" style="font-family: 'Crimson Text', serif;">
                      {{ truncate(note.content, 60) }}
                    </p>
                  </div>

                  <!-- Rhythm -->
                  <div v-else-if="note.note_type === 'rhythm'" class="relative flex items-center gap-6">
                    <div class="text-4xl font-light text-[#2C1810] font-mono">
                      {{ note.metadata.time_signature || '4/4' }}
                    </div>
                    <p class="text-[#5C4A36]" style="font-family: 'Crimson Text', serif;">
                      {{ truncate(note.content, 50) }}
                    </p>
                  </div>

                  <!-- Score -->
                  <div v-else-if="note.note_type === 'score'" class="relative flex items-center gap-4">
                    <div class="w-16 h-12 border border-dashed border-[#8B7355] flex items-center justify-center bg-[#FDF8F0]">
                      <span class="text-2xl text-[#C4B59D]">𝄞</span>
                    </div>
                    <p class="text-[#5C4A36]" style="font-family: 'Crimson Text', serif;">
                      {{ truncate(note.content, 60) }}
                    </p>
                  </div>
                </div>

                <!-- Catalog number (like work number) -->
                <div class="mt-4 pt-3 border-t border-dashed border-[#C4B59D] flex justify-between items-center">
                  <span class="text-[9px] text-[#A69580] font-mono">
                    Op. {{ note.id.substring(0, 6).toUpperCase() }}
                  </span>
                  <span class="text-sm text-[#8B7355] opacity-0 group-hover:opacity-100 transition-opacity">
                    открыть →
                  </span>
                </div>

                <!-- Bar line right -->
                <div class="absolute right-0 top-0 bottom-0 w-0.5 bg-[#C4B59D] group-hover:bg-[#2C1810] transition-colors"></div>
              </div>
            </article>
          </div>

          <!-- End of movement (final bar) -->
          <div class="flex justify-end mt-6">
            <div class="flex gap-1">
              <div class="w-0.5 h-8 bg-[#C4B59D]"></div>
              <div class="w-1 h-8 bg-[#2C1810]"></div>
            </div>
          </div>
        </section>
      </div>

      <!-- Empty state -->
      <div v-else-if="!store.loading" class="text-center py-24">
        <div class="text-8xl text-[#C4B59D] mb-6">𝄞</div>
        <h3 class="text-2xl text-[#5C4A36] mb-4" style="font-family: 'Crimson Text', serif;">
          Партитура пуста
        </h3>
        <p class="text-[#8B7355] mb-8">Начните записывать свои музыкальные идеи</p>
        <button
          @click="openCreate"
          class="px-8 py-3 bg-[#2C1810] text-[#FDF8F0] hover:bg-[#1a0f0a] transition-all"
          style="font-family: 'Crimson Text', serif;"
        >
          Первая запись
        </button>
      </div>

      <!-- Loading -->
      <div v-if="store.loading" class="text-center py-24">
        <div class="text-6xl text-[#8B7355] animate-pulse mb-4">𝄞</div>
        <p class="text-[#8B7355] text-sm uppercase tracking-widest">Загрузка партитуры...</p>
      </div>

      <!-- Load more -->
      <div v-if="store.hasMore && notesByYear.length > 0" class="text-center mt-16">
        <div class="inline-flex items-center gap-4">
          <div class="h-px w-16 bg-[#C4B59D]"></div>
          <button
            @click="store.loadMore()"
            :disabled="store.loading"
            class="px-8 py-3 border border-[#8B7355] text-[#5C4A36] hover:bg-[#8B7355] hover:text-white transition-all disabled:opacity-50"
            style="font-family: 'Crimson Text', serif;"
          >
            {{ store.loading ? 'Загрузка...' : 'Следующие страницы' }}
          </button>
          <div class="h-px w-16 bg-[#C4B59D]"></div>
        </div>
      </div>
    </main>

    <!-- Footer (like score ending) -->
    <footer class="relative z-10 py-12 border-t-2 border-[#8B7355] mt-16">
      <div class="max-w-5xl mx-auto px-8 text-center">

        <!-- Fine marking -->
        <div class="text-3xl italic text-[#8B7355] mb-4" style="font-family: 'Crimson Text', serif;">
          Fine
        </div>

        <p class="text-sm text-[#8B7355]">
          Gmazz — Собрание сочинений © 1974—{{ new Date().getFullYear() }}
        </p>
        <p class="text-xs text-[#A69580] mt-2 italic" style="font-family: 'Crimson Text', serif;">
          «Аранжировка — это искусство услышать то, чего ещё нет»
        </p>

        <!-- Final double bar -->
        <div class="flex justify-center gap-1 mt-8">
          <div class="w-0.5 h-6 bg-[#8B7355]"></div>
          <div class="w-1.5 h-6 bg-[#2C1810]"></div>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
