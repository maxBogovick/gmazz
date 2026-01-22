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
const visibleNotes = ref<string[]>([]);
const scrollY = ref(0);

const collections = [
  {
    type: undefined as NoteType | undefined,
    title: 'Полная Коллекция',
    subtitle: 'Complete Works',
    era: '1974—2026',
    icon: '♪'
  },
  {
    type: 'score' as NoteType,
    title: 'Партитуры',
    subtitle: 'Original Scores',
    era: 'Manuscript Hall',
    icon: '𝄞'
  },
  {
    type: 'phrase' as NoteType,
    title: 'Фразировка',
    subtitle: 'Phrase Studies',
    era: 'Recording Archive',
    icon: '♫'
  },
  {
    type: 'harmony' as NoteType,
    title: 'Гармония',
    subtitle: 'Harmonic Analysis',
    era: 'Theory Cabinet',
    icon: '♯'
  },
  {
    type: 'rhythm' as NoteType,
    title: 'Ритмы',
    subtitle: 'Rhythmic Patterns',
    era: 'Metronome Room',
    icon: '𝅘𝅥𝅮'
  },
  {
    type: 'thought' as NoteType,
    title: 'Заметки',
    subtitle: 'Personal Reflections',
    era: 'Private Journal',
    icon: '✍'
  },
];

const headerVisible = computed(() => scrollY.value > 200);

onMounted(async () => {
  await store.fetchNotes();
  animateNotes();
  window.addEventListener('scroll', handleScroll);
});

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll);
});

function handleScroll() {
  scrollY.value = window.scrollY;
}

function animateNotes() {
  visibleNotes.value = [];
  store.filteredNotes.forEach((note, index) => {
    setTimeout(() => {
      visibleNotes.value.push(note.id);
    }, index * 80);
  });
}

function openNote(note: Note) {
  router.push({ name: 'note', params: { id: note.id } });
}

function openCreate() {
  router.push({ name: 'create' });
}

function setCollection(type: NoteType | undefined) {
  store.setFilter(type);
  visibleNotes.value = [];
  setTimeout(animateNotes, 200);
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

const currentCollection = computed(() =>
    collections.find(c => c.type === store.filter.note_type) || collections[0]
);
</script>

<template>
  <div class="min-h-screen bg-[#FAF7F2] text-[#3D3428] font-serif overflow-x-hidden">

    <!-- Floating Header -->
    <header
        class="fixed top-0 left-0 right-0 z-50 transition-all duration-700"
        :class="headerVisible ? 'bg-[#FAF7F2]/95 backdrop-blur-md border-b border-[#D4CAB5] shadow-sm' : 'bg-transparent'"
    >
      <div class="max-w-7xl mx-auto px-6 lg:px-12 py-6 flex items-center justify-between">

        <button @click="setCollection(undefined)" class="group flex items-baseline gap-3">
          <span class="text-3xl font-light tracking-tight text-[#5C5245] group-hover:text-[#A67C00] transition-colors duration-300">
            Gmazz
          </span>
          <span class="text-[9px] uppercase tracking-[0.25em] text-[#8B7E6A] font-sans mt-2">
            Est. 1974
          </span>
        </button>

        <button
            @click="openCreate"
            class="px-5 py-2.5 bg-[#A67C00] text-white hover:bg-[#B8860B] text-xs uppercase tracking-widest font-sans transition-all duration-300 rounded"
        >
          Новая Запись
        </button>
      </div>
    </header>

    <!-- Hero Exhibition -->
    <section class="relative min-h-[85vh] flex items-center justify-center overflow-hidden pt-20">

      <!-- Decorative Background -->
      <div class="absolute inset-0">
        <div class="absolute top-1/4 left-1/4 w-96 h-96 bg-[#A67C00]/5 rounded-full blur-[120px]"></div>
        <div class="absolute bottom-1/4 right-1/4 w-96 h-96 bg-[#8B5A2B]/5 rounded-full blur-[120px]"></div>
      </div>

      <!-- Staff Lines Decoration -->
      <div class="absolute left-0 right-0 top-1/2 -translate-y-1/2 opacity-[0.06] pointer-events-none">
        <div v-for="i in 5" :key="i" class="h-px bg-[#8B7E6A] mb-6"></div>
      </div>

      <div class="relative z-10 max-w-6xl mx-auto px-6 lg:px-12 text-center space-y-12">

        <!-- Main Title -->
        <div class="space-y-8">
          <div class="inline-block px-6 py-2 border border-[#D4CAB5] text-[10px] uppercase tracking-[0.35em] text-[#A67C00] font-sans rounded">
            Джазовый Аранжировщик • Композитор • Педагог
          </div>

          <h1 class="text-5xl sm:text-6xl lg:text-8xl xl:text-9xl font-light leading-[1.05] tracking-tight">
            <span class="block text-[#5C5245]">Пятьдесят Лет</span>
            <span class="block text-[#A67C00] mt-2">
              в Джазе
            </span>
          </h1>

          <div class="max-w-3xl mx-auto">
            <p class="text-lg sm:text-xl lg:text-2xl text-[#8B7E6A] font-light leading-relaxed italic">
              "Аранжировка — это искусство услышать то, чего ещё нет,<br class="hidden sm:block"/> и записать то, что невозможно объяснить словами"
            </p>
          </div>
        </div>

        <!-- Stats -->
        <div class="flex flex-wrap justify-center gap-10 lg:gap-20 pt-12">
          <div class="text-center group">
            <div class="text-4xl sm:text-5xl lg:text-6xl font-extralight text-[#A67C00] mb-3 group-hover:text-[#B8860B] transition-colors">51</div>
            <div class="text-[10px] uppercase tracking-[0.2em] text-[#8B7E6A] font-sans">год творчества</div>
          </div>
          <div class="text-center group">
            <div class="text-4xl sm:text-5xl lg:text-6xl font-extralight text-[#A67C00] mb-3 group-hover:text-[#B8860B] transition-colors">{{ store.notes.length || '—' }}</div>
            <div class="text-[10px] uppercase tracking-[0.2em] text-[#8B7E6A] font-sans">работ в архиве</div>
          </div>
          <div class="text-center group">
            <div class="text-4xl sm:text-5xl lg:text-6xl font-extralight text-[#A67C00] mb-3 group-hover:text-[#B8860B] transition-colors">∞</div>
            <div class="text-[10px] uppercase tracking-[0.2em] text-[#8B7E6A] font-sans">вдохновения</div>
          </div>
        </div>

        <!-- Scroll Indicator -->
        <div class="pt-16 animate-bounce">
          <div class="flex flex-col items-center gap-3">
            <span class="text-[9px] uppercase tracking-[0.3em] text-[#A89F8B] font-sans">Исследовать архив</span>
            <div class="w-px h-12 bg-gradient-to-b from-[#A67C00]/60 to-transparent"></div>
          </div>
        </div>
      </div>
    </section>

    <!-- Biography Section -->
    <section class="relative py-24 lg:py-32 border-y border-[#E0D9C8] bg-[#F5F1E8]">
      <div class="max-w-5xl mx-auto px-6 lg:px-12">
        <div class="grid lg:grid-cols-2 gap-16 items-center">

          <!-- Left: Quote/Image placeholder -->
          <div class="relative">
            <div class="aspect-[4/5] bg-white border border-[#D4CAB5] rounded-lg flex items-center justify-center shadow-sm">
              <div class="text-center p-8">
                <div class="text-8xl text-[#A67C00]/30 mb-6">𝄞</div>
                <blockquote class="text-lg text-[#5C5245] italic leading-relaxed">
                  "Каждая нота должна дышать. Каждый аккорд — рассказывать историю."
                </blockquote>
              </div>
            </div>
            <!-- Decorative frame corner -->
            <div class="absolute -top-3 -left-3 w-12 h-12 border-t-2 border-l-2 border-[#A67C00]/40 rounded-tl-lg"></div>
            <div class="absolute -bottom-3 -right-3 w-12 h-12 border-b-2 border-r-2 border-[#A67C00]/40 rounded-br-lg"></div>
          </div>

          <!-- Right: Biography text -->
          <div class="space-y-8">
            <div>
              <div class="flex items-center gap-4 mb-6">
                <div class="h-px flex-1 bg-gradient-to-r from-[#D4CAB5] to-transparent"></div>
                <span class="text-[10px] uppercase tracking-[0.3em] text-[#A67C00] font-sans">Об Авторе</span>
              </div>
              <h2 class="text-3xl lg:text-4xl font-light text-[#3D3428] mb-6 leading-tight">
                Полвека служения<br/>
                <span class="text-[#A67C00]">джазовому искусству</span>
              </h2>
            </div>

            <div class="space-y-5 text-[#5C5245] font-light leading-relaxed">
              <p>
                С 1974 года — непрерывный путь через оркестровые партитуры,
                камерные ансамбли и биг-бэнды. Каждая аранжировка — это диалог
                между традицией и новаторством.
              </p>
              <p>
                Этот архив — личное хранилище идей, гармонических открытий
                и ритмических экспериментов, собранных за более чем пятьдесят лет
                практики.
              </p>
            </div>

            <div class="pt-4 flex flex-wrap gap-6 text-sm">
              <div class="flex items-center gap-3">
                <div class="w-2 h-2 bg-[#A67C00] rounded-full"></div>
                <span class="text-[#8B7E6A]">Биг-бэнд аранжировки</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="w-2 h-2 bg-[#A67C00] rounded-full"></div>
                <span class="text-[#8B7E6A]">Оркестровые партитуры</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="w-2 h-2 bg-[#A67C00] rounded-full"></div>
                <span class="text-[#8B7E6A]">Педагогика</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Collection Navigation -->
    <nav class="sticky top-[73px] z-40 bg-[#FAF7F2]/95 backdrop-blur-md border-y border-[#E0D9C8]">
      <div class="max-w-7xl mx-auto px-6 lg:px-12">
        <div class="flex overflow-x-auto scrollbar-hide">
          <button
              v-for="(coll, idx) in collections"
              :key="idx"
              @click="setCollection(coll.type)"
              class="flex-shrink-0 group relative px-8 py-6 transition-all duration-300"
              :class="store.filter.note_type === coll.type ? 'text-[#A67C00]' : 'text-[#8B7E6A] hover:text-[#5C5245]'"
          >
            <div class="flex flex-col items-center gap-2">
              <span class="text-2xl">{{ coll.icon }}</span>
              <span class="text-sm font-medium">{{ coll.title }}</span>
              <span class="text-[10px] uppercase tracking-wider font-sans opacity-60">{{ coll.subtitle }}</span>
            </div>

            <!-- Active Indicator -->
            <div
                class="absolute bottom-0 left-1/2 -translate-x-1/2 h-0.5 bg-[#A67C00] transition-all duration-500"
                :class="store.filter.note_type === coll.type ? 'w-3/4 opacity-100' : 'w-0 opacity-0'"
            ></div>
          </button>
        </div>
      </div>
    </nav>

    <!-- Main Gallery -->
    <main class="relative z-10 max-w-7xl mx-auto px-6 lg:px-12 py-24">

      <!-- Collection Header -->
      <div v-if="!store.loading" class="mb-20 text-center fade-in">
        <div class="inline-flex items-center gap-4 mb-6">
          <div class="h-px w-12 bg-[#D4CAB5]"></div>
          <span class="text-xs uppercase tracking-[0.3em] text-[#A67C00] font-sans">
            {{ currentCollection.era }}
          </span>
          <div class="h-px w-12 bg-[#D4CAB5]"></div>
        </div>

        <h2 class="text-4xl lg:text-5xl font-light mb-4 text-[#3D3428]">
          {{ currentCollection.title }}
        </h2>
        <p class="text-[#8B7E6A] italic">{{ currentCollection.subtitle }}</p>
      </div>

      <!-- Works Grid -->
      <div v-if="store.filteredNotes.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 lg:gap-10">
        <TransitionGroup name="exhibit">
          <article
              v-for="note in store.filteredNotes"
              :key="note.id"
              v-show="visibleNotes.includes(note.id)"
              @click="openNote(note)"
              class="group cursor-pointer"
          >
            <!-- Frame -->
            <div class="relative bg-white border border-[#E0D9C8] rounded-lg p-1 transition-all duration-500 group-hover:border-[#A67C00]/40 group-hover:shadow-lg">

              <!-- Inner Mat -->
              <div class="bg-[#FFFEFA] rounded p-6 min-h-[280px] flex flex-col relative overflow-hidden">

                <!-- Corner Ornaments -->
                <div class="absolute top-3 left-3 w-4 h-4 border-t border-l border-[#D4CAB5]"></div>
                <div class="absolute top-3 right-3 w-4 h-4 border-t border-r border-[#D4CAB5]"></div>
                <div class="absolute bottom-3 left-3 w-4 h-4 border-b border-l border-[#D4CAB5]"></div>
                <div class="absolute bottom-3 right-3 w-4 h-4 border-b border-r border-[#D4CAB5]"></div>

                <!-- Catalog Number -->
                <div class="flex justify-between items-center mb-4 text-[10px] font-sans uppercase tracking-widest text-[#A89F8B]">
                  <span>№ {{ note.id.substring(0, 6) }}</span>
                  <span>{{ new Date(note.created_at).getFullYear() }}</span>
                </div>

                <!-- Content -->
                <div class="flex-1 relative z-10">
                  <component :is="getCardComponent(note.note_type)" :note="note" />
                </div>
              </div>
            </div>

            <!-- Label Plate -->
            <div class="mt-4 text-center">
              <div class="text-sm text-[#8B7E6A] group-hover:text-[#A67C00] transition-colors">
                {{ collections.find(c => c.type === note.note_type)?.title || 'Untitled' }}
              </div>
            </div>
          </article>
        </TransitionGroup>
      </div>

      <!-- Load More -->
      <div v-if="store.hasMore && store.filteredNotes.length > 0" class="flex flex-col items-center mt-24 gap-8">
        <div class="flex items-center gap-4">
          <div class="h-px w-16 bg-gradient-to-r from-transparent to-[#D4CAB5]"></div>
          <span class="text-xs uppercase tracking-widest text-[#A89F8B] font-sans">Показать ещё</span>
          <div class="h-px w-16 bg-gradient-to-l from-transparent to-[#D4CAB5]"></div>
        </div>

        <button
            @click="store.loadMore()"
            :disabled="store.loading"
            class="px-8 py-3 border border-[#D4CAB5] text-[#A67C00] hover:bg-[#F5F1E8] hover:border-[#A67C00] text-sm uppercase tracking-widest font-sans transition-all duration-300 disabled:opacity-30 rounded"
        >
          {{ store.loading ? 'Загрузка...' : 'Загрузить ещё' }}
        </button>
      </div>

      <!-- Empty State -->
      <div v-else-if="!store.loading && store.filteredNotes.length === 0" class="min-h-[50vh] flex flex-col items-center justify-center text-center fade-in">
        <div class="w-32 h-32 border border-[#D4CAB5] rounded-lg flex items-center justify-center mb-8 text-5xl text-[#A67C00]/40">
          ♪
        </div>
        <h3 class="text-2xl font-light mb-4 text-[#5C5245]">Коллекция Пуста</h3>
        <button
            @click="openCreate"
            class="text-[#A67C00] hover:text-[#B8860B] text-sm uppercase tracking-widest border-b border-[#D4CAB5] hover:border-[#A67C00] pb-1 transition-all font-sans"
        >
          Добавить Первую Работу
        </button>
      </div>

      <!-- Loading -->
      <div v-else-if="store.loading" class="min-h-[60vh] flex items-center justify-center">
        <div class="text-center space-y-4">
          <div class="text-3xl text-[#A67C00] animate-pulse">𝄞</div>
          <div class="text-sm uppercase tracking-widest text-[#A89F8B] font-sans">Загрузка архива...</div>
        </div>
      </div>

    </main>

    <!-- Footer Archive Info -->
    <footer class="border-t border-[#E0D9C8] bg-[#F5F1E8] py-16 mt-16">
      <div class="max-w-7xl mx-auto px-6 lg:px-12">
        <div class="grid md:grid-cols-3 gap-12 mb-12">

          <div>
            <h4 class="text-sm uppercase tracking-widest text-[#A67C00] mb-4 font-sans">О Коллекции</h4>
            <p class="text-sm text-[#8B7E6A] leading-relaxed font-light">
              Личный архив джазового аранжировщика и композитора. Каждая партитура, каждая заметка — часть полувекового путешествия через большие оркестры, квартеты и бесконечные поиски идеального звучания.
            </p>
          </div>

          <div>
            <h4 class="text-sm uppercase tracking-widest text-[#A67C00] mb-4 font-sans">Хронология</h4>
            <div class="space-y-2 text-sm text-[#8B7E6A] font-light">
              <div class="flex justify-between border-b border-[#E0D9C8] pb-2">
                <span>Начало карьеры</span>
                <span class="text-[#A67C00]">1974</span>
              </div>
              <div class="flex justify-between border-b border-[#E0D9C8] pb-2">
                <span>Активная деятельность</span>
                <span class="text-[#A67C00]">51 год</span>
              </div>
              <div class="flex justify-between border-b border-[#E0D9C8] pb-2">
                <span>Архив обновлён</span>
                <span class="text-[#A67C00]">{{ new Date().getFullYear() }}</span>
              </div>
            </div>
          </div>

          <div>
            <h4 class="text-sm uppercase tracking-widest text-[#A67C00] mb-4 font-sans">Навигация</h4>
            <div class="space-y-3">
              <button
                  v-for="coll in collections.slice(1)"
                  :key="coll.title"
                  @click="setCollection(coll.type)"
                  class="block text-sm text-[#8B7E6A] hover:text-[#A67C00] transition-colors text-left font-light"
              >
                {{ coll.title }}
              </button>
            </div>
          </div>

        </div>

        <div class="pt-8 border-t border-[#E0D9C8] text-center">
          <p class="text-xs uppercase tracking-[0.25em] text-[#A89F8B] font-sans">
            © {{ new Date().getFullYear() }} Gmazz Musical Archive — Персональный архив
          </p>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.exhibit-enter-active,
.exhibit-leave-active {
  transition: all 0.7s cubic-bezier(0.4, 0, 0.2, 1);
}

.exhibit-enter-from {
  opacity: 0;
  transform: translateY(30px) scale(0.96);
}

.exhibit-leave-to {
  opacity: 0;
  transform: translateY(-30px) scale(0.96);
}

.fade-in {
  animation: fadeIn 1.2s ease-out forwards;
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

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
