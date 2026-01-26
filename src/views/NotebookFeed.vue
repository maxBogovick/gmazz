<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { uploadFileToServer, getSetting, setSetting } from '../api/server';
import { getAssetPath } from '../api/notes';
import type { Note, NoteType } from '../types';

const router = useRouter();
const store = useNotesStore();
const mouseX = ref(0.5);
const mouseY = ref(0.5);
const scrollY = ref(0);
const time = ref(0);
const isReducedMotion = ref(false);
const isSmallScreen = ref(false);
const isLightAmbient = ref(false);
let motionMedia: MediaQueryList | null = null;
let isAnimating = false;

// Profile photo
const isTauri = !!(window as any).__TAURI_INTERNALS__;
const profilePhotoUrl = ref<string>('');
const isPhotoHovered = ref(false);
const isUploadingPhoto = ref(false);
const photoInputRef = ref<HTMLInputElement | null>(null);

// Editable Profile Data
const profileData = ref({
  firstName: 'Сергей',
  lastName: 'Гмыря',
  role: 'Музыкант',
  description: 'Мысли, гармонии, мелодические фразы и партитуры — живой архив музыкальных идей',
  quote: 'Музыка — это то, что происходит между нотами',
  startYear: '1974'
});

async function loadProfileData() {
  try {
    const [fname, lname, role, desc, quote, year] = await Promise.all([
      getSetting('profile_firstname'),
      getSetting('profile_lastname'),
      getSetting('profile_role'),
      getSetting('profile_description'),
      getSetting('profile_quote'),
      getSetting('archive_start_year')
    ]);
    
    if (fname) profileData.value.firstName = fname;
    if (lname) profileData.value.lastName = lname;
    if (role) profileData.value.role = role;
    if (desc) profileData.value.description = desc;
    if (quote) profileData.value.quote = quote;
    if (year) profileData.value.startYear = year;
  } catch (e) {
    console.error('Failed to load profile data', e);
  }
}

async function saveProfileField(key: string, value: string) {
  if (!isTauri) return;
  try {
    await setSetting(key, value);
  } catch (e) {
    console.error(`Failed to save ${key}`, e);
  }
}

// Load profile photo from server
async function loadProfilePhoto() {
  try {
    const url = await getSetting('profile_photo');
    if (url) {
      // Use getAssetPath to get proper URL with authentication
      profilePhotoUrl.value = await getAssetPath(url);
    }
  } catch (error) {
    console.error('Failed to load profile photo:', error);
  }
}

function triggerPhotoUpload() {
  photoInputRef.value?.click();
}

async function handlePhotoChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  if (!file.type.startsWith('image/')) {
    alert('Пожалуйста, выберите изображение');
    return;
  }

  isUploadingPhoto.value = true;
  try {
    // Upload file to server
    const url = await uploadFileToServer(file);
    // Save URL in settings
    await setSetting('profile_photo', url);
    // Get displayable URL via getAssetPath
    profilePhotoUrl.value = await getAssetPath(url);
  } catch (error) {
    console.error('Failed to upload photo:', error);
    alert('Не удалось загрузить фото');
  } finally {
    isUploadingPhoto.value = false;
    input.value = '';
  }
}


const typeConfig: Record<NoteType, { name: string; icon: string; accent: string; bg: string; gradient: string; frequency: number }> = {
  thought: {
    name: 'Мысль',
    icon: '✦',
    accent: '#C4956A',
    bg: 'linear-gradient(135deg, #FDF8F3 0%, #F9F1E8 100%)',
    gradient: 'linear-gradient(135deg, rgba(196, 149, 106, 0.15) 0%, rgba(196, 149, 106, 0.05) 100%)',
    frequency: 261.63 // C4
  },
  harmony: {
    name: 'Гармония',
    icon: '♮',
    accent: '#7B9E87',
    bg: 'linear-gradient(135deg, #F5F9F6 0%, #EBF4EE 100%)',
    gradient: 'linear-gradient(135deg, rgba(123, 158, 135, 0.15) 0%, rgba(123, 158, 135, 0.05) 100%)',
    frequency: 329.63 // E4
  },
  phrase: {
    name: 'Фраза',
    icon: '𝄞',
    accent: '#8B7BA8',
    bg: 'linear-gradient(135deg, #F8F6FA 0%, #F0ECF5 100%)',
    gradient: 'linear-gradient(135deg, rgba(139, 123, 168, 0.15) 0%, rgba(139, 123, 168, 0.05) 100%)',
    frequency: 392.00 // G4
  },
  rhythm: {
    name: 'Ритм',
    icon: '◈',
    accent: '#B8856E',
    bg: 'linear-gradient(135deg, #FBF6F4 0%, #F6EDE8 100%)',
    gradient: 'linear-gradient(135deg, rgba(184, 133, 110, 0.15) 0%, rgba(184, 133, 110, 0.05) 100%)',
    frequency: 440.00 // A4
  },
  score: {
    name: 'Партитура',
    icon: '𝄚',
    accent: '#6B8FAD',
    bg: 'linear-gradient(135deg, #F5F8FA 0%, #EAF1F6 100%)',
    gradient: 'linear-gradient(135deg, rgba(107, 143, 173, 0.15) 0%, rgba(107, 143, 173, 0.05) 100%)',
    frequency: 523.25 // C5
  },
};

let animationFrame: number;

onMounted(async () => {
  await Promise.all([
    store.fetchNotes(),
    loadProfilePhoto(),
    loadProfileData()
  ]);
  const savedAmbient = localStorage.getItem('gmazz_light_ambient');
  isLightAmbient.value = savedAmbient === '1';
  updateMotionPrefs();
  window.addEventListener('resize', updateMotionPrefs, { passive: true });
  if (motionMedia) {
    motionMedia.addEventListener('change', updateMotionPrefs);
  }
  window.addEventListener('scroll', handleScroll, { passive: true });
  syncAmbientMotion();
});

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('scroll', handleScroll);
  window.removeEventListener('resize', updateMotionPrefs);
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

function handleScroll() {
  scrollY.value = window.scrollY;
}

function updateMotionPrefs() {
  if (!motionMedia) {
    motionMedia = window.matchMedia('(prefers-reduced-motion: reduce)');
  }
  isReducedMotion.value = motionMedia.matches;
  isSmallScreen.value = window.innerWidth < 768;
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

function toggleLightAmbient() {
  isLightAmbient.value = !isLightAmbient.value;
  localStorage.setItem('gmazz_light_ambient', isLightAmbient.value ? '1' : '0');
  syncAmbientMotion();
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
    month: 'short',
    year: 'numeric'
  });
}

function truncate(text: string, len: number): string {
  return text.length > len ? text.slice(0, len) + '…' : text;
}

const ambientStyle = computed(() => {
  if (isReducedMotion.value || isLightAmbient.value) {
    return {
      background: 'linear-gradient(180deg, #FAF6F2 0%, #F6F1EC 35%, #F3EDE6 70%, #F0E8E0 100%)'
    };
  }
  return {
    background: `
      radial-gradient(ellipse 100% 70% at ${30 + mouseX.value * 25}% ${20 + mouseY.value * 25}%, rgba(255, 225, 190, 0.25) 0%, transparent 65%),
      radial-gradient(ellipse 80% 60% at ${70 - mouseX.value * 20}% ${60 + mouseY.value * 20}%, rgba(210, 190, 170, 0.18) 0%, transparent 65%),
      radial-gradient(ellipse 60% 50% at ${50 + Math.sin(time.value) * 15}% ${50 + Math.cos(time.value * 0.8) * 15}%, rgba(255, 240, 220, 0.15) 0%, transparent 55%),
      linear-gradient(180deg, #FAF6F2 0%, #F6F1EC 30%, #F3EDE6 70%, #F0E8E0 100%)
    `
  };
});

const headerOpacity = computed(() => Math.min(scrollY.value / 100, 1));
const parallaxOffset = computed(() => scrollY.value * 0.5);

function getCardSize(index: number) {
  const pattern = index % 7;
  return {
    cols: pattern === 0 ? 'col-span-12 md:col-span-8' :
        pattern === 5 || pattern === 6 ? 'col-span-12 md:col-span-6' :
            'col-span-12 md:col-span-4',
    minHeight: pattern === 0 ? 'min-h-[320px]' : 'min-h-[240px]',
    truncateLength: pattern === 0 ? 300 : 160
  };
}

// Musical notation elements for decoration
const musicalNotes = ['𝅝', '𝅗𝅥', '𝅘𝅥', '𝅘𝅥𝅮', '𝅘𝅥𝅯', '♩', '♪', '♫', '♬'];
const floatingNotesCount = computed(() => (isReducedMotion.value || isSmallScreen.value ? 12 : 40));

function getCardChips(note: Note): string[] {
  const chips: string[] = [];

  if (note.note_type === 'thought') {
    if (note.metadata?.tags?.length) {
      chips.push(...note.metadata.tags.slice(0, 2).map(tag => `#${tag}`));
    }
  } else if (note.note_type === 'harmony') {
    if (note.metadata?.chord_symbol) chips.push(`аккорд ${note.metadata.chord_symbol}`);
    if (note.metadata?.key) chips.push(`тон ${note.metadata.key}`);
  } else if (note.note_type === 'phrase') {
    if (note.metadata?.file_path) chips.push('аудио');
    if (note.metadata?.duration) chips.push(`${note.metadata.duration}с`);
  } else if (note.note_type === 'rhythm') {
    if (note.metadata?.time_signature) chips.push(`размер ${note.metadata.time_signature}`);
    if (note.metadata?.mood) chips.push(note.metadata.mood);
  } else if (note.note_type === 'score') {
    if (note.metadata?.key) chips.push(`тон ${note.metadata.key}`);
    if (note.metadata?.audio_path) chips.push('аудио');
  }

  return chips.slice(0, 3);
}
</script>

<template>
  <div class="min-h-screen overflow-x-hidden" :style="ambientStyle">

    <!-- Enhanced Atmospheric Layers with Musical Theme -->
    <div v-if="!isReducedMotion && !isLightAmbient" class="fixed inset-0 pointer-events-none overflow-hidden">
      <!-- Primary warm glow -->
      <div
          class="absolute -top-1/3 -left-1/4 w-[1200px] h-[1200px] rounded-full opacity-60 will-change-transform blur-[100px]"
          style="background: radial-gradient(circle, rgba(255, 230, 200, 0.9) 0%, rgba(255, 215, 170, 0.5) 35%, transparent 70%);"
          :style="{ transform: `translate3d(${mouseX * 50}px, ${mouseY * 50}px, 0) scale(${1 + Math.sin(time) * 0.08})` }"
      />

      <!-- Secondary ambient glow -->
      <div
          class="absolute -bottom-1/3 -right-1/4 w-[900px] h-[900px] rounded-full opacity-50 will-change-transform blur-[100px]"
          style="background: radial-gradient(circle, rgba(210, 180, 160, 0.7) 0%, rgba(190, 160, 140, 0.4) 35%, transparent 70%);"
          :style="{ transform: `translate3d(${-mouseX * 35}px, ${-mouseY * 35}px, 0) scale(${1 + Math.cos(time * 0.7) * 0.08})` }"
      />

      <!-- Tertiary accent glow -->
      <div
          class="absolute top-1/2 left-1/2 w-[700px] h-[700px] rounded-full opacity-30 will-change-transform blur-[80px]"
          style="background: radial-gradient(circle, rgba(220, 200, 180, 0.6) 0%, transparent 60%);"
          :style="{ transform: `translate3d(${Math.sin(time * 0.5) * 100}px, ${Math.cos(time * 0.3) * 100}px, 0)` }"
      />

      <!-- Soft gradient overlays -->
      <div
          class="absolute top-1/4 right-0 w-full h-[500px] opacity-40"
          style="background: linear-gradient(90deg, transparent 0%, rgba(250, 240, 230, 0.95) 50%, transparent 100%);"
          :style="{ transform: `translateX(${Math.sin(time * 0.4) * 30}px)` }"
      />

      <!-- Multiple light shafts -->
      <div
          class="absolute left-1/5 top-0 w-[2px] h-full opacity-15 blur-sm"
          style="background: linear-gradient(180deg, transparent 0%, rgba(255, 220, 180, 0.9) 20%, rgba(255, 220, 180, 0.9) 80%, transparent 100%);"
          :style="{ transform: `translateX(${mouseX * 120}px)` }"
      />
      <div
          class="absolute right-1/3 top-0 w-[2px] h-full opacity-15 blur-sm"
          style="background: linear-gradient(180deg, transparent 0%, rgba(210, 190, 170, 0.9) 25%, rgba(210, 190, 170, 0.9) 75%, transparent 100%);"
          :style="{ transform: `translateX(${-mouseX * 80}px)` }"
      />
    </div>

    <!-- Enhanced Floating Musical Notes -->
    <div v-if="!isReducedMotion && !isSmallScreen && !isLightAmbient" class="fixed inset-0 pointer-events-none overflow-hidden">
      <div
          v-for="i in floatingNotesCount"
          :key="i"
          class="absolute will-change-transform font-serif"
          :class="i % 4 === 0 ? 'text-3xl' : i % 4 === 1 ? 'text-2xl' : i % 4 === 2 ? 'text-xl' : 'text-lg'"
          :style="{
          left: `${(i * 7) % 100}%`,
          top: `${(i * 9 + 15) % 100}%`,
          color: i % 3 === 0 ? 'rgba(196, 149, 106, 0.08)' : i % 3 === 1 ? 'rgba(123, 158, 135, 0.08)' : 'rgba(139, 123, 168, 0.08)',
          animation: `float-musical-${i % 3} ${8 + i % 6}s ease-in-out infinite`,
          animationDelay: `${i * 0.15}s`,
          textShadow: i % 5 === 0 ? '0 0 30px rgba(255, 220, 180, 0.4)' : 'none',
          filter: i % 6 === 0 ? 'blur(0.5px)' : 'none'
        }"
      >
        {{ musicalNotes[i % musicalNotes.length] }}
      </div>
    </div>

    <!-- Subtle grain texture -->
    <div class="fixed inset-0 pointer-events-none opacity-[0.025] mix-blend-overlay"
         style="background-image: url('data:image/svg+xml,%3Csvg viewBox=&quot;0 0 200 200&quot; xmlns=&quot;http://www.w3.org/2000/svg&quot;%3E%3Cfilter id=&quot;noise&quot;%3E%3CfeTurbulence type=&quot;fractalNoise&quot; baseFrequency=&quot;1.2&quot; numOctaves=&quot;3&quot; /%3E%3C/filter%3E%3Crect width=&quot;100%&quot; height=&quot;100%&quot; filter=&quot;url(%23noise)&quot; /%3E%3C/svg%3E');" />

    <!-- Header with enhanced glassmorphism -->
    <header
        class="fixed top-0 left-0 right-0 z-50 backdrop-blur-2xl transition-all duration-700 border-b"
        :style="{
        backgroundColor: `rgba(250, 246, 242, ${0.65 + headerOpacity * 0.25})`,
        borderColor: `rgba(200, 180, 160, ${0.15 + headerOpacity * 0.25})`,
        boxShadow: headerOpacity > 0.5 ? '0 8px 40px rgba(0,0,0,0.06), 0 2px 8px rgba(200,180,160,0.1)' : 'none'
      }"
    >
      <div class="max-w-7xl mx-auto px-8 lg:px-16 h-20 flex items-center justify-between">
        <div class="flex items-center gap-4 cursor-pointer group" @click="router.push('/')">
          <div class="relative w-11 h-11 rounded-2xl bg-gradient-to-br from-amber-50 via-orange-50 to-amber-100 flex items-center justify-center text-amber-700 text-xl shadow-xl group-hover:shadow-2xl transition-all duration-500 group-hover:scale-110 group-hover:rotate-12 overflow-hidden">
            <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-amber-200/50 to-orange-200/50 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
            <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-white/40 to-transparent" />
            <span class="relative z-10 font-serif">𝄞</span>
          </div>
          <div>
            <span class="text-xl font-semibold text-stone-700 group-hover:text-amber-700 transition-colors duration-300 tracking-tight">Gmazz</span>
            <span class="text-[11px] text-stone-400 block -mt-0.5 tracking-[0.25em] uppercase font-medium">личный архив</span>
          </div>
        </div>

        <nav class="flex items-center gap-3">
          <button
              @click="toggleLightAmbient"
              class="relative text-sm text-stone-500 hover:text-amber-700 transition-all duration-300 flex items-center gap-2.5 px-4 py-2.5 rounded-xl hover:bg-white/70 overflow-hidden group backdrop-blur-sm"
              :aria-pressed="isLightAmbient"
              title="Лёгкий фон"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-amber-200/0 via-amber-200/60 to-amber-200/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000" />
            <span class="text-amber-600 relative text-base">{{ isLightAmbient ? '☀︎' : '☾' }}</span>
            <span class="relative font-medium">{{ isLightAmbient ? 'полный фон' : 'лёгкий фон' }}</span>
          </button>
          <button
              @click="openRandomNote"
              class="relative text-sm text-stone-500 hover:text-amber-700 transition-all duration-300 flex items-center gap-2.5 px-5 py-2.5 rounded-xl hover:bg-white/70 overflow-hidden group backdrop-blur-sm"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-amber-200/0 via-amber-200/60 to-amber-200/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000" />
            <span class="text-amber-600 animate-sparkle relative text-base">✦</span>
            <span class="relative font-medium">случайная</span>
          </button>
          <button
              @click="openCreate"
              class="relative text-sm text-stone-700 bg-gradient-to-br from-white/95 to-stone-50/95 hover:from-white hover:to-stone-50 px-7 py-2.5 rounded-xl shadow-lg hover:shadow-2xl transition-all duration-300 active:scale-95 overflow-hidden group backdrop-blur-sm border border-stone-200/50"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-amber-50/0 via-amber-50/80 to-amber-50/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000" />
            <span class="relative font-semibold tracking-tight">+ новая запись</span>
          </button>
        </nav>
      </div>
    </header>

    <!-- Hidden file input for photo upload -->
    <input
        ref="photoInputRef"
        type="file"
        accept="image/*"
        class="hidden"
        @change="handlePhotoChange"
    />

    <!-- Enhanced Hero Section -->
    <section class="relative pt-40 pb-28 px-8 lg:px-16" :style="{ transform: `translateY(${-parallaxOffset * 0.3}px)` }">
      <div class="max-w-7xl mx-auto">
        <div class="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-12 lg:gap-16">
          <!-- Mobile Photo (shown only on small screens) -->
          <div class="lg:hidden flex justify-center mb-8">
            <div
                class="relative group/photo-mobile"
                @click="isTauri ? triggerPhotoUpload() : null"
            >
              <!-- Decorative frame -->
              <div class="absolute -inset-3 bg-gradient-to-br from-amber-200/40 to-amber-50/40 rounded-[2rem] blur-xl opacity-80" />

              <!-- Photo frame -->
              <div
                  class="relative w-48 h-56 rounded-[1.5rem] overflow-hidden shadow-xl"
                  :class="profilePhotoUrl ? '' : 'bg-gradient-to-br from-stone-100 via-amber-50 to-stone-100'"
              >
                <template v-if="profilePhotoUrl">
                  <img :src="profilePhotoUrl" alt="Сергей Гмыря" class="w-full h-full object-cover" />
                  <div class="absolute inset-0 bg-gradient-to-t from-stone-900/20 via-transparent to-transparent pointer-events-none" />
                </template>
                <template v-else>
                  <div class="absolute inset-0 flex flex-col items-center justify-center text-stone-400">
                    <div class="text-6xl mb-2 opacity-30 font-serif">𝄞</div>
                    <span class="text-[11px] uppercase tracking-widest opacity-50">Фото</span>
                  </div>
                </template>

                <!-- Frame border -->
                <div class="absolute inset-0 rounded-[1.5rem] border-2 border-amber-200/50 pointer-events-none" />

                <!-- Edit indicator for Tauri -->
                <div
                    v-if="isTauri"
                    class="absolute inset-0 bg-stone-900/0 active:bg-stone-900/40 transition-all duration-300 flex items-center justify-center"
                >
                  <div class="opacity-0 active:opacity-100 transition-opacity">
                    <div class="w-10 h-10 rounded-xl bg-white/20 backdrop-blur-sm flex items-center justify-center">
                      <span v-if="isUploadingPhoto" class="animate-spin">⟳</span>
                      <span v-else>📷</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Left: Text Content -->
          <div class="max-w-3xl flex-1">
            <div class="flex items-center gap-5 mb-10 overflow-hidden">
              <div class="text-5xl text-amber-400 animate-pulse-slow drop-shadow-lg">❧</div>
              <div class="h-[2px] flex-1 relative overflow-hidden rounded-full">
                <div class="absolute inset-0 bg-gradient-to-r from-amber-300 via-amber-200 to-transparent" />
                <div class="absolute inset-0 bg-gradient-to-r from-amber-400 via-amber-300 to-transparent translate-x-[-100%] animate-shimmer" />
              </div>
            </div>

            <h1 class="text-7xl lg:text-8xl font-light text-stone-800 leading-[0.95] mb-10 tracking-tight flex flex-col items-start gap-2">
              <div class="flex items-baseline gap-4 flex-wrap">
                <input
                  v-if="isTauri"
                  v-model="profileData.firstName"
                  @change="saveProfileField('profile_firstname', profileData.firstName)"
                  class="bg-transparent border-b border-transparent hover:border-amber-300 focus:border-amber-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/60 transition-all min-w-[1ch] w-auto max-w-full"
                  :style="{ width: profileData.firstName.length + 'ch' }"
                  aria-label="Имя"
                />
                <span v-else class="inline-block hover:text-amber-700 transition-colors duration-700 drop-shadow-sm">{{ profileData.firstName }}</span>

                <input
                  v-if="isTauri"
                  v-model="profileData.lastName"
                  @change="saveProfileField('profile_lastname', profileData.lastName)"
                  class="bg-transparent border-b border-transparent hover:border-amber-300 focus:border-amber-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/60 transition-all min-w-[1ch] w-auto max-w-full"
                  :style="{ width: profileData.lastName.length + 'ch' }"
                  aria-label="Фамилия"
                />
                <span v-else class="inline-block hover:text-amber-700 transition-colors duration-700 drop-shadow-sm">{{ profileData.lastName }}</span>
              </div>
              
              <input
                v-if="isTauri"
                v-model="profileData.role"
                @change="saveProfileField('profile_role', profileData.role)"
                class="text-stone-400 mt-4 text-6xl lg:text-7xl bg-transparent border-b border-transparent hover:border-stone-300 focus:border-stone-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-300/60 transition-all w-full"
                aria-label="Роль"
              />
              <span v-else class="block text-stone-400 mt-4 text-6xl lg:text-7xl hover:text-stone-500 transition-colors duration-700">{{ profileData.role }}</span>
            </h1>

            <div class="text-2xl lg:text-[26px] text-stone-500 leading-relaxed max-w-2xl">
              <textarea
                v-if="isTauri"
                v-model="profileData.description"
                @change="saveProfileField('profile_description', profileData.description)"
                rows="3"
                class="w-full bg-transparent border-l-2 border-transparent hover:border-amber-300 focus:border-amber-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/60 transition-all resize-none"
                aria-label="Описание"
              ></textarea>
              <p v-else>
                {{ profileData.description }}
              </p>
            </div>

            <!-- Enhanced Stats -->
            <div class="flex items-center gap-16 mt-16">
              <div class="group text-center relative cursor-default">
                <div class="absolute inset-0 bg-gradient-to-br from-amber-300/25 to-transparent rounded-3xl blur-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                <span class="block text-6xl font-light text-stone-700 group-hover:text-amber-700 transition-all duration-700 group-hover:scale-110 relative drop-shadow-sm">
                  {{ store.notes.length || '—' }}
                </span>
                <span class="text-[11px] text-stone-400 uppercase tracking-[0.25em] mt-3 block font-semibold">записей</span>
              </div>
              <div class="w-[2px] h-14 bg-gradient-to-b from-transparent via-stone-300 to-transparent rounded-full" />
              <div class="group text-center relative cursor-default">
                <div class="absolute inset-0 bg-gradient-to-br from-amber-300/25 to-transparent rounded-3xl blur-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                <span class="block text-6xl font-light text-stone-700 group-hover:text-amber-700 transition-all duration-700 group-hover:scale-110 relative drop-shadow-sm">5</span>
                <span class="text-[11px] text-stone-400 uppercase tracking-[0.25em] mt-3 block font-semibold">типов</span>
              </div>
              <div class="w-[2px] h-14 bg-gradient-to-b from-transparent via-stone-300 to-transparent rounded-full" />
              <div class="group text-center relative cursor-default">
                <div class="absolute inset-0 bg-gradient-to-br from-amber-300/25 to-transparent rounded-3xl blur-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
                <span class="block text-6xl font-light text-stone-700 group-hover:text-amber-700 transition-all duration-700 group-hover:scale-110 relative drop-shadow-sm">В наше время</span>
                <span class="text-[11px] text-stone-400 uppercase tracking-[0.25em] mt-3 block font-semibold">начало</span>
              </div>
            </div>
          </div>

          <!-- Right: Profile Photo (desktop) -->
          <div class="relative flex-shrink-0 hidden lg:block">
            <!-- Photo Container -->
            <div
                class="relative group/photo"
                @mouseenter="isPhotoHovered = true"
                @mouseleave="isPhotoHovered = false"
            >
              <!-- Decorative frame layers -->
              <div class="absolute -inset-4 bg-gradient-to-br from-amber-200/40 via-orange-100/30 to-amber-50/40 rounded-[2.5rem] blur-2xl opacity-80 group-hover/photo:opacity-100 transition-opacity duration-700" />
              <div class="absolute -inset-2 bg-gradient-to-br from-amber-100/60 to-stone-100/60 rounded-[2rem] opacity-60" />

              <!-- Main photo frame -->
              <div
                  class="relative w-72 h-80 lg:w-80 lg:h-[22rem] rounded-[1.75rem] overflow-hidden shadow-2xl transition-all duration-700 group-hover/photo:shadow-3xl"
                  :class="profilePhotoUrl ? '' : 'bg-gradient-to-br from-stone-100 via-amber-50 to-stone-100'"
              >
                <!-- Photo or placeholder -->
                <template v-if="profilePhotoUrl">
                  <img
                      :src="profilePhotoUrl"
                      alt="Сергей Гмыря"
                      class="w-full h-full object-cover transition-transform duration-1000 group-hover/photo:scale-105"
                  />
                  <!-- Subtle vignette overlay -->
                  <div class="absolute inset-0 bg-gradient-to-t from-stone-900/20 via-transparent to-stone-900/5 pointer-events-none" />
                </template>

                <!-- Placeholder when no photo -->
                <template v-else>
                  <div class="absolute inset-0 flex flex-col items-center justify-center text-stone-400">
                    <div class="text-8xl mb-4 opacity-30 font-serif">𝄞</div>
                    <span class="text-sm uppercase tracking-widest opacity-50">Фото</span>
                  </div>
                </template>

                <!-- Golden frame border -->
                <div class="absolute inset-0 rounded-[1.75rem] border-2 border-amber-200/50 pointer-events-none" />

                <!-- Decorative corner ornaments -->
                <div class="absolute top-3 left-3 w-8 h-8 border-t-2 border-l-2 border-amber-300/60 rounded-tl-xl pointer-events-none" />
                <div class="absolute top-3 right-3 w-8 h-8 border-t-2 border-r-2 border-amber-300/60 rounded-tr-xl pointer-events-none" />
                <div class="absolute bottom-3 left-3 w-8 h-8 border-b-2 border-l-2 border-amber-300/60 rounded-bl-xl pointer-events-none" />
                <div class="absolute bottom-3 right-3 w-8 h-8 border-b-2 border-r-2 border-amber-300/60 rounded-br-xl pointer-events-none" />

                <!-- Light reflection effect -->
                <div
                    class="absolute inset-0 opacity-0 group-hover/photo:opacity-100 transition-opacity duration-700 pointer-events-none"
                    :style="{
                      background: `linear-gradient(${135 + mouseX * 30}deg, rgba(255,255,255,0.15) 0%, transparent 50%)`
                    }"
                />

                <!-- Edit overlay (only in Tauri mode) -->
                <div
                    v-if="isTauri"
                    class="absolute inset-0 bg-stone-900/0 group-hover/photo:bg-stone-900/40 transition-all duration-500 flex items-center justify-center cursor-pointer"
                    @click="triggerPhotoUpload"
                >
                  <div
                      class="opacity-0 group-hover/photo:opacity-100 transition-all duration-500 transform translate-y-4 group-hover/photo:translate-y-0"
                  >
                    <div class="flex flex-col items-center gap-3 text-white">
                      <div class="w-14 h-14 rounded-2xl bg-white/20 backdrop-blur-sm flex items-center justify-center border border-white/30 shadow-lg">
                        <span v-if="isUploadingPhoto" class="text-xl animate-spin">⟳</span>
                        <span v-else class="text-2xl">📷</span>
                      </div>
                      <span class="text-sm font-medium tracking-wide uppercase">
                        {{ isUploadingPhoto ? 'Загрузка...' : (profilePhotoUrl ? 'Изменить' : 'Добавить фото') }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Decorative musical notes around frame -->
              <div class="absolute -top-6 -right-2 text-3xl text-amber-300/40 animate-float font-serif" style="animation-delay: 0.2s;">♪</div>
              <div class="absolute -bottom-4 -left-4 text-4xl text-amber-300/30 animate-float font-serif" style="animation-delay: 0.8s;">♫</div>
              <div class="absolute top-1/2 -right-8 text-2xl text-amber-200/40 animate-float font-serif" style="animation-delay: 1.4s;">𝅘𝅥𝅮</div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Notes Collection -->
    <section class="relative px-8 lg:px-16 pb-32">
      <div class="max-w-7xl mx-auto">

        <div class="flex items-center justify-between mb-16">
            <h2 class="text-[11px] uppercase tracking-[0.3em] text-stone-400 font-bold">Коллекция работ</h2>
          <div class="h-[2px] flex-1 mx-10 relative overflow-hidden rounded-full">
            <div class="absolute inset-0 bg-gradient-to-r from-stone-300 via-stone-200 to-transparent" />
            <div class="absolute inset-0 bg-gradient-to-r from-amber-300 to-transparent translate-x-[-100%] animate-shimmer-slow" />
          </div>
        </div>

        <!-- Ultra Enhanced Bento Grid -->
        <div v-if="store.filteredNotes.length > 0" class="grid grid-cols-12 gap-7 lg:gap-8">
          <article
              v-for="(note, index) in store.filteredNotes"
              :key="note.id"
              @click="openNote(note)"
              class="group cursor-pointer perspective-1000"
              :class="getCardSize(index).cols"
          >
            <div
                class="relative h-full rounded-[28px] p-8 lg:p-10 transition-all duration-700 ease-out group-hover:shadow-2xl overflow-hidden preserve-3d will-change-transform"
                :style="{
                background: typeConfig[note.note_type].bg,
                transform: 'translateZ(0)'
              }"
                :class="getCardSize(index).minHeight"
            >
              <!-- Enhanced layered glows -->
              <div
                  class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-all duration-1000 pointer-events-none blur-3xl"
                  :style="{
                  background: `radial-gradient(circle at ${mouseX * 100}% ${mouseY * 100}%, ${typeConfig[note.note_type].accent}40 0%, ${typeConfig[note.note_type].accent}15 35%, transparent 70%)`
                }"
              />

              <div
                  class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-all duration-700 pointer-events-none"
                  :style="{
                  background: typeConfig[note.note_type].gradient
                }"
              />

              <!-- Animated border with musical pulse -->
              <div class="absolute inset-0 rounded-[28px] opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none overflow-hidden">
                <div
                    class="absolute inset-0 border-2 rounded-[28px]"
                    :style="{
                    borderColor: typeConfig[note.note_type].accent + '40',
                    animation: `pulse-border 2s ease-in-out infinite`
                  }"
                />
              </div>

              <!-- 3D floating icon with rotation -->
              <div
                  class="absolute top-7 right-7 text-6xl lg:text-7xl opacity-[0.06] group-hover:opacity-25 transition-all duration-1000 will-change-transform font-serif"
                  :style="{
                  color: typeConfig[note.note_type].accent,
                  transform: `translateZ(40px) rotateY(${mouseX * 25 - 12.5}deg) rotateX(${mouseY * -25 + 12.5}deg) scale(${1 + Math.sin(time + index * 0.5) * 0.12})`
                }"
              >
                {{ typeConfig[note.note_type].icon }}
              </div>

              <!-- Dynamic light reflection -->
              <div
                  class="absolute inset-0 opacity-0 group-hover:opacity-40 transition-opacity duration-1000 pointer-events-none"
                  :style="{
                  background: `linear-gradient(${mouseX * 180}deg, transparent 0%, ${typeConfig[note.note_type].accent}15 50%, transparent 100%)`
                }"
              />

              <!-- Card Content -->
              <div class="relative z-10 h-full flex flex-col">
                <!-- Enhanced Header -->
                <div class="flex items-center gap-3.5 mb-7">
                  <span
                      class="w-12 h-12 rounded-2xl flex items-center justify-center text-lg group-hover:scale-125 group-hover:rotate-12 transition-all duration-700 shadow-lg group-hover:shadow-2xl relative overflow-hidden font-serif"
                      :style="{
                      backgroundColor: typeConfig[note.note_type].accent + '35',
                      color: typeConfig[note.note_type].accent
                    }"
                  >
                    <span class="absolute inset-0 bg-gradient-to-br from-white/30 to-transparent" />
                    <span class="relative drop-shadow-sm">{{ typeConfig[note.note_type].icon }}</span>
                  </span>
                  <span class="text-[11px] font-bold uppercase tracking-[0.18em] group-hover:tracking-[0.25em] transition-all duration-500" :style="{ color: typeConfig[note.note_type].accent }">
                    {{ typeConfig[note.note_type].name }}
                  </span>
                  <span class="text-[11px] text-stone-400 ml-auto font-mono tabular-nums group-hover:text-stone-600 transition-colors">
                    {{ formatDate(note.created_at) }}
                  </span>
                </div>

                <div v-if="getCardChips(note).length" class="flex flex-wrap gap-2 mb-6">
                  <span
                      v-for="(chip, chipIndex) in getCardChips(note)"
                      :key="note.id + '-' + chipIndex"
                      class="inline-flex items-center gap-1 px-3 py-1 rounded-full text-[11px] uppercase tracking-[0.18em] font-semibold bg-white/70 border border-white/70 shadow-sm"
                      :style="{ color: typeConfig[note.note_type].accent, borderColor: typeConfig[note.note_type].accent + '35' }"
                  >
                    {{ chip }}
                  </span>
                </div>

                <!-- Main Content with Enhanced Styling -->
                <div class="flex-1">
                  <!-- Thought -->
                  <div v-if="note.note_type === 'thought'" class="h-full flex flex-col">
                    <p class="text-2xl lg:text-[28px] text-stone-600 leading-relaxed font-light flex-1 group-hover:text-stone-800 transition-colors duration-700">
                      «{{ truncate(note.content, getCardSize(index).truncateLength) }}»
                    </p>
                  </div>

                  <!-- Harmony -->
                  <div v-else-if="note.note_type === 'harmony'" class="h-full flex flex-col">
                    <div class="flex-1 bg-white/80 rounded-2xl p-7 backdrop-blur-sm border border-white/60 group-hover:bg-white/95 group-hover:shadow-xl group-hover:border-white/80 transition-all duration-700">
                      <pre class="font-mono text-sm text-stone-600 whitespace-pre-wrap leading-loose group-hover:text-stone-800 transition-colors">{{ truncate(note.content, getCardSize(index).truncateLength) }}</pre>
                    </div>
                    <div v-if="note.metadata?.chord_symbol" class="mt-6 flex items-center gap-3">
                      <span class="text-[11px] text-stone-400 uppercase tracking-wider font-semibold">аккорд:</span>
                      <span class="text-xl font-mono font-bold text-stone-700 px-4 py-1.5 bg-white/70 rounded-xl shadow-md">{{ note.metadata.chord_symbol }}</span>
                    </div>
                  </div>

                  <!-- Phrase -->
                  <div v-else-if="note.note_type === 'phrase'" class="h-full flex flex-col">
                    <div class="flex items-center gap-6 mb-7">
                      <div
                          class="w-[70px] h-[70px] rounded-2xl flex items-center justify-center text-white shadow-xl group-hover:scale-125 group-hover:rotate-12 transition-all duration-700 cursor-pointer relative overflow-hidden"
                          :style="{ backgroundColor: typeConfig[note.note_type].accent }"
                      >
                        <div class="absolute inset-0 bg-gradient-to-br from-white/40 to-transparent" />
                        <span class="text-2xl relative animate-pulse-slow drop-shadow-md">▶</span>
                      </div>
                      <!-- Enhanced waveform -->
                      <div class="flex-1 flex items-center gap-[3px] h-14">
                        <div
                            v-for="j in 36"
                            :key="j"
                            class="flex-1 rounded-full transition-all duration-500 group-hover:opacity-100"
                            :style="{
                            height: `${25 + Math.sin(j * 0.35 + time) * 65}%`,
                            backgroundColor: typeConfig[note.note_type].accent + '70',
                            opacity: 0.6 + Math.sin(time * 2.5 + j * 0.25) * 0.3,
                            transform: `scaleY(${1 + Math.sin(time * 3.5 + j * 0.15) * 0.35})`
                          }"
                        />
                      </div>
                    </div>
                    <p class="text-stone-500 leading-relaxed flex-1 group-hover:text-stone-700 transition-colors duration-700 text-lg">{{ truncate(note.content, 130) }}</p>
                  </div>

                  <!-- Rhythm -->
                  <div v-else-if="note.note_type === 'rhythm'" class="h-full flex flex-col">
                    <div class="flex items-start gap-10">
                      <div
                          class="text-7xl font-light leading-none group-hover:scale-110 transition-transform duration-700"
                          :style="{
                          color: typeConfig[note.note_type].accent,
                          textShadow: `0 0 25px ${typeConfig[note.note_type].accent}50`,
                          filter: 'drop-shadow(0 4px 8px rgba(0,0,0,0.1))'
                        }"
                      >
                        {{ note.metadata?.time_signature || '4/4' }}
                      </div>
                      <div class="flex-1">
                        <p class="text-stone-500 leading-relaxed group-hover:text-stone-700 transition-colors duration-700 text-lg">{{ truncate(note.content, 150) }}</p>
                        <div v-if="note.metadata?.mood" class="mt-6">
                          <span class="text-[11px] px-5 py-2 rounded-full bg-white/90 text-stone-600 font-bold shadow-md border border-stone-200/50">
                            {{ note.metadata.mood }}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- Score -->
                  <div v-else-if="note.note_type === 'score'" class="h-full flex flex-col">
                    <div class="flex-1 bg-white/70 rounded-2xl p-7 border border-stone-200/80 backdrop-blur-sm group-hover:bg-white/90 group-hover:shadow-xl group-hover:border-stone-300/80 transition-all duration-700">
                      <div class="relative h-28 mb-6">
                        <div
                            v-for="j in 5"
                            :key="j"
                            class="absolute left-0 right-0 h-[2px] bg-stone-300/80 transition-all duration-700 group-hover:bg-stone-400/90 rounded-full"
                            :style="{
                            top: `${j * 22}%`,
                            transform: `translateY(${Math.sin(time + j * 0.8) * 2}px)`,
                            boxShadow: `0 0 8px ${typeConfig[note.note_type].accent}20`
                          }"
                        />
                        <div
                            class="absolute left-10 top-1/2 -translate-y-1/2 text-5xl text-stone-400 group-hover:text-stone-600 transition-all duration-700 font-serif drop-shadow-md"
                            :style="{
                            transform: `translateY(-50%) scale(${1 + Math.sin(time) * 0.08})`,
                            color: typeConfig[note.note_type].accent + '80'
                          }"
                        >
                          𝄞
                        </div>
                      </div>
                      <p class="text-stone-500 text-base leading-relaxed group-hover:text-stone-700 transition-colors duration-700">{{ truncate(note.content, 130) }}</p>
                    </div>
                    <div v-if="note.metadata?.key" class="mt-6 text-[11px] text-stone-400">
                      Тональность: <span class="text-stone-700 font-bold text-sm">{{ note.metadata.key }}</span>
                    </div>
                  </div>
                </div>

                <!-- Enhanced Footer -->
                <div class="flex items-center justify-between mt-7 pt-7 border-t border-stone-200/80 group-hover:border-stone-300/90 transition-colors duration-700">
                  <span class="text-[11px] text-stone-400 font-mono tracking-wider group-hover:text-stone-500 transition-colors duration-500 tabular-nums">#{{ note.id.substring(0, 8) }}</span>
                  <span
                      class="text-[11px] font-bold opacity-60 group-hover:opacity-100 transition-all duration-700 flex items-center gap-2.5 group-hover:translate-x-2"
                      :style="{ color: typeConfig[note.note_type].accent }"
                  >
                    открыть <span class="text-base animate-pulse-slow">→</span>
                  </span>
                </div>
              </div>
            </div>
          </article>
        </div>

        <!-- Enhanced Empty State -->
        <div v-else-if="!store.loading" class="text-center py-48">
          <div class="inline-block p-20 rounded-[2.5rem] bg-white/80 backdrop-blur-2xl shadow-2xl relative overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-amber-100/25 to-transparent" />
            <div class="absolute top-0 right-0 w-40 h-40 bg-amber-200/25 rounded-full blur-3xl animate-pulse-slow" />
            <div class="absolute bottom-0 left-0 w-40 h-40 bg-orange-200/25 rounded-full blur-3xl animate-pulse-slow" style="animation-delay: 1s;" />

            <div class="relative">
              <div class="text-9xl text-amber-300 mb-10 inline-block animate-float font-serif drop-shadow-lg" :style="{ transform: `scale(${1 + Math.sin(time) * 0.1})` }">𝄞</div>
              <h3 class="text-4xl font-light text-stone-700 mb-5 tracking-tight">Архив пуст</h3>
              <p class="text-stone-500 mb-12 max-w-md leading-relaxed text-xl mx-auto">
                Создайте первую запись, чтобы начать формировать коллекцию музыкальных идей
              </p>
              <button
                  @click="openCreate"
                  class="relative text-base text-stone-700 bg-white hover:bg-stone-50 px-12 py-5 rounded-2xl shadow-xl hover:shadow-2xl transition-all duration-500 active:scale-95 overflow-hidden group border border-stone-200/50"
              >
                <div class="absolute inset-0 bg-gradient-to-r from-amber-50/0 via-amber-50/90 to-amber-50/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1200" />
                <span class="relative font-bold tracking-tight">+ создать запись</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Loading -->
        <div v-if="store.loading" class="text-center py-48">
          <div class="inline-block">
            <div class="relative w-20 h-20 mb-8">
              <div class="absolute inset-0 border-[3px] border-amber-200 rounded-full animate-ping opacity-25" />
              <div class="absolute inset-0 border-[3px] border-amber-300 border-t-amber-700 rounded-full animate-spin" />
            </div>
            <p class="text-sm text-stone-400 font-bold tracking-[0.25em] uppercase">загрузка...</p>
          </div>
        </div>

        <!-- Load More -->
        <div v-if="store.hasMore && store.filteredNotes.length > 0" class="text-center mt-24">
          <button
              @click="store.loadMore()"
              :disabled="store.loading"
              class="relative text-sm text-stone-600 border-2 border-stone-300 px-14 py-5 rounded-2xl hover:bg-white/80 hover:border-stone-400 hover:shadow-2xl transition-all duration-500 disabled:opacity-50 disabled:cursor-not-allowed active:scale-95 overflow-hidden group backdrop-blur-sm"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-stone-50/0 via-stone-50/80 to-stone-50/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1200" />
            <span class="relative font-semibold tracking-tight">показать ещё</span>
          </button>
        </div>
      </div>
    </section>

    <!-- Enhanced Footer -->
    <footer class="relative px-8 lg:px-16 py-24 border-t border-stone-200/80 bg-gradient-to-b from-transparent to-stone-50/70 overflow-hidden">
      <div class="absolute inset-0 opacity-40">
        <div class="absolute top-0 left-1/4 w-80 h-80 bg-amber-200/25 rounded-full blur-[100px]" />
        <div class="absolute bottom-0 right-1/4 w-80 h-80 bg-orange-200/25 rounded-full blur-[100px]" />
      </div>

      <div class="max-w-7xl mx-auto relative">
        <div class="flex flex-col md:flex-row items-center justify-between gap-10">
          <div class="flex items-center gap-6 group cursor-pointer">
            <span class="text-5xl text-amber-400 group-hover:scale-125 group-hover:rotate-12 transition-all duration-700 font-serif drop-shadow-lg" :style="{ transform: `scale(${1 + Math.sin(time * 0.5) * 0.08})` }">𝄞</span>
            <div>
              <span class="text-stone-700 font-bold text-2xl group-hover:text-amber-700 transition-colors duration-500 tracking-tight">Gmazz</span>
              <div class="text-[11px] text-stone-400 block tracking-[0.25em] uppercase mt-1 font-semibold flex items-center gap-1">
                <span>архив</span>
                <input
                  v-if="isTauri"
                  v-model="profileData.startYear"
                  @change="saveProfileField('archive_start_year', profileData.startYear)"
                  class="bg-transparent w-12 border-b border-stone-200 focus:border-amber-500 outline-none focus-visible:ring-2 focus-visible:ring-amber-200/60 text-center"
                  aria-label="Год начала архива"
                />
                <span v-else>{{ profileData.startYear }}</span>
                <span>—{{ new Date().getFullYear() }}</span>
              </div>
            </div>
          </div>
          <div class="text-lg text-stone-500 italic text-center md:text-right max-w-lg leading-relaxed relative">
            <span class="absolute -top-6 -left-6 text-5xl text-amber-300/40 font-serif">"</span>
            <textarea
              v-if="isTauri"
              v-model="profileData.quote"
              @change="saveProfileField('profile_quote', profileData.quote)"
              rows="2"
              class="w-full bg-transparent border-none focus:ring-0 focus-visible:ring-2 focus-visible:ring-amber-200/60 text-right resize-none outline-none"
              aria-label="Цитата"
            ></textarea>
            <span v-else>{{ profileData.quote }}</span>
            <span class="absolute -bottom-6 -right-6 text-5xl text-amber-300/40 font-serif">"</span>
          </div>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
@keyframes float-musical-0 {
  0%, 100% {
    transform: translate3d(0, 0, 0) rotate(0deg);
    opacity: 0.3;
  }
  25% {
    transform: translate3d(20px, -30px, 0) rotate(90deg);
    opacity: 0.7;
  }
  50% {
    transform: translate3d(-10px, -20px, 0) rotate(180deg);
    opacity: 0.4;
  }
  75% {
    transform: translate3d(10px, -40px, 0) rotate(270deg);
    opacity: 0.6;
  }
}

@keyframes float-musical-1 {
  0%, 100% {
    transform: translate3d(0, 0, 0) rotate(0deg);
    opacity: 0.4;
  }
  33% {
    transform: translate3d(-15px, -25px, 0) rotate(120deg);
    opacity: 0.8;
  }
  66% {
    transform: translate3d(15px, -35px, 0) rotate(240deg);
    opacity: 0.5;
  }
}

@keyframes float-musical-2 {
  0%, 100% {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.35;
  }
  50% {
    transform: translate3d(12px, -45px, 0) scale(1.4);
    opacity: 0.75;
  }
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

@keyframes shimmer-slow {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(200%);
  }
}

@keyframes pulse-slow {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.75;
    transform: scale(1.05);
  }
}

@keyframes pulse-border {
  0%, 100% {
    opacity: 0.3;
  }
  50% {
    opacity: 0.6;
  }
}

@keyframes sparkle {
  0%, 100% {
    opacity: 1;
    transform: scale(1) rotate(0deg);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.3) rotate(180deg);
  }
}

@keyframes float {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-20px);
  }
}

.animate-shimmer {
  animation: shimmer 3s infinite;
}

.animate-shimmer-slow {
  animation: shimmer-slow 8s infinite;
}

.animate-pulse-slow {
  animation: pulse-slow 3s ease-in-out infinite;
}

.animate-sparkle {
  animation: sparkle 2s ease-in-out infinite;
}

.animate-float {
  animation: float 3s ease-in-out infinite;
}

.shadow-3xl {
  box-shadow: 0 35px 60px -15px rgba(0, 0, 0, 0.15), 0 15px 30px -10px rgba(0, 0, 0, 0.1);
}

.perspective-1000 {
  perspective: 1000px;
}

.preserve-3d {
  transform-style: preserve-3d;
}

* {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.group {
  transition: transform 0.7s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.will-change-transform {
  will-change: transform;
}

.group > div {
  backface-visibility: hidden;
  transform: translateZ(0);
}

/* Smooth scroll behavior */
html {
  scroll-behavior: smooth;
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 10px;
}

::-webkit-scrollbar-track {
  background: rgba(245, 240, 235, 0.5);
}

::-webkit-scrollbar-thumb {
  background: rgba(196, 149, 106, 0.3);
  border-radius: 5px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(196, 149, 106, 0.5);
}
</style>
