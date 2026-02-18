<script setup lang="ts">
import { onMounted, ref, computed, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { getSetting, isTauri, setSetting, uploadFileAndGetId } from '../api/server';
import { getAssetPath } from '../api/notes';
import type { Note, NoteType } from '../types';
import NoteCard from '../components/NoteCard.vue';

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

const isTauriEnv = !!(window as any).__TAURI_INTERNALS__;
const canEditSettings = computed(() => isTauriEnv);
const profilePhotoUrl = ref<string>('');
const isPhotoHovered = ref(false);
const isUploadingPhoto = ref(false);
const photoInputRef = ref<HTMLInputElement | null>(null);

const profileData = ref({
  firstName: 'Сергей',
  lastName: 'Гмыря',
  role: 'Музыкант',
  description: 'Мысли, гармонии, мелодические фразы и партитуры — живой архив музыкальных идей',
  quote: 'Музыка — это то, что происходит между нотами',
  startYear: 'Большой опыт'
});

const maestroBlock = ref({
  title: 'Архив Маэстро',
  headline: 'Живая хроника творчества — от первых рукописей до зрелых партитур',
  description: 'Здесь собраны мысли, гармонии, фразы и партитуры музыканта с огромным опытом. Каждый лист — след развития стиля, ремесла и внутреннего слуха.',
  cta: '+ создать запись'
});

const genres = [
  {
    name: 'Джаз', icon: '🎷',
    sub: 'Импровизация & свинг',
    desc: 'Свобода выражения через сложные гармонии и импровизацию',
    color: '#C17A3A', secondary: '#8B5A2B',
    gradient: 'linear-gradient(135deg, #D4A574 0%, #B87333 45%, #8B5A2B 100%)',
    glow: 'rgba(193, 122, 58, 0.4)',
    pattern: '♩ ♪ ♫'
  },
  {
    name: 'Госпел', icon: '🎹',
    sub: 'Дух & вдохновение',
    desc: 'Духовная музыка, наполненная эмоциями и верой',
    color: '#2D7A6B', secondary: '#1A5647',
    gradient: 'linear-gradient(135deg, #4FA89A 0%, #2D7A6B 45%, #1A5647 100%)',
    glow: 'rgba(45, 122, 107, 0.4)',
    pattern: '♬ 𝄞 ♮'
  },
  {
    name: 'Акапелла', icon: '🎵',
    sub: 'Голос без границ',
    desc: 'Чистота вокала, гармония голосов в идеальном созвучии',
    color: '#7A4BA3', secondary: '#5A3578',
    gradient: 'linear-gradient(135deg, #9B6BC7 0%, #7A4BA3 45%, #5A3578 100%)',
    glow: 'rgba(122, 75, 163, 0.4)',
    pattern: '♪ ♫ 𝅘𝅥𝅮'
  },
  {
    name: 'Классика', icon: '🎼',
    sub: 'Вечная традиция',
    desc: 'Бессмертные произведения, проверенные временем',
    color: '#2A5C8F', secondary: '#1A3D5F',
    gradient: 'linear-gradient(135deg, #4682B4 0%, #2A5C8F 45%, #1A3D5F 100%)',
    glow: 'rgba(42, 92, 143, 0.4)',
    pattern: '𝄚 ♮ ◈'
  },
];

const featuredNoteIds = ref<string[]>([]);
const isEditingFeatured = ref(false);
const currentPlayingId = ref<string | null>(null);
const audioPlayer = ref<HTMLAudioElement | null>(null);

async function loadProfileData() {
  try {
    const [fname, lname, role, desc, quote, year] = await Promise.all([
      getSetting('profile_firstname'), getSetting('profile_lastname'),
      getSetting('profile_role'), getSetting('profile_description'),
      getSetting('profile_quote'), getSetting('archive_start_year')
    ]);
    if (fname) profileData.value.firstName = fname;
    if (lname) profileData.value.lastName = lname;
    if (role) profileData.value.role = role;
    if (desc) profileData.value.description = desc;
    if (quote) profileData.value.quote = quote;
    if (year) profileData.value.startYear = year;
  } catch (e) { console.error('Failed to load profile data', e); }
}

async function saveProfileField(key: string, value: string) {
  if (!canEditSettings.value) return;
  try { await setSetting(key, value); } catch (e) { console.error(`Failed to save ${key}`, e); }
}

async function loadMaestroBlock() {
  try {
    const [title, headline, description, cta, featured] = await Promise.all([
      getSetting('maestro_title'), getSetting('maestro_headline'),
      getSetting('maestro_description'), getSetting('maestro_cta'),
      getSetting('featured_note_ids')
    ]);
    if (title) maestroBlock.value.title = title;
    if (headline) maestroBlock.value.headline = headline;
    if (description) maestroBlock.value.description = description;
    if (cta) maestroBlock.value.cta = cta;
    if (featured) {
      try {
        const parsed = JSON.parse(featured);
        if (Array.isArray(parsed)) featuredNoteIds.value = parsed.filter((id) => typeof id === 'string');
      } catch { featuredNoteIds.value = []; }
    }
  } catch (e) { console.error('Failed to load maestro block', e); }
}

async function saveMaestroField(key: string, value: string) {
  if (!canEditSettings.value) return;
  try { await setSetting(key, value); } catch (e) { console.error(`Failed to save ${key}`, e); }
}

async function persistFeaturedNotes() {
  if (!canEditSettings.value) return;
  try { await setSetting('featured_note_ids', JSON.stringify(featuredNoteIds.value)); } catch (e) { console.error('Failed to save featured notes', e); }
}

async function loadProfilePhoto() {
  try {
    const url = await getSetting('profile_photo');
    if (url) profilePhotoUrl.value = await getAssetPath(url);
  } catch (error) { console.error('Failed to load profile photo:', error); }
}

function triggerPhotoUpload() { photoInputRef.value?.click(); }

async function handlePhotoChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  if (!file.type.startsWith('image/')) { alert('Пожалуйста, выберите изображение'); return; }
  isUploadingPhoto.value = true;
  try {
    const fileId = await uploadFileAndGetId(file);
    await setSetting('profile_photo', fileId);
    profilePhotoUrl.value = await getAssetPath(fileId);
  } catch (error) { console.error('Failed to upload photo:', error); alert('Не удалось загрузить фото'); }
  finally { isUploadingPhoto.value = false; input.value = ''; }
}

const typeConfig: Record<NoteType, { name: string; icon: string; accent: string; light: string; dark: string; tag: string }> = {
  thought: { name: 'Мысль',      icon: '✦',  accent: '#B8722E', light: '#FDF4E8', dark: '#6B3E10', tag: '#FAE4C0' },
  phrase:  { name: 'Фраза',      icon: '𝄞',  accent: '#6B4A8E', light: '#F2EDF8', dark: '#3A2058', tag: '#D8C4F5' },
  score:   { name: 'Партитура',  icon: '𝄚',  accent: '#2A5C90', light: '#E8F0FA', dark: '#102E50', tag: '#B5CFEE' },
};

let animationFrame: number;

onMounted(async () => {
  await Promise.all([store.fetchNotes(), loadProfilePhoto(), loadProfileData(), loadMaestroBlock()]);
  const savedAmbient = isTauri() ? await getSetting('gmazz_light_ambient') : localStorage.getItem('gmazz_light_ambient');
  isLightAmbient.value = savedAmbient === '1';
  updateMotionPrefs();
  window.addEventListener('resize', updateMotionPrefs, { passive: true });
  if (motionMedia) motionMedia.addEventListener('change', updateMotionPrefs);
  window.addEventListener('scroll', handleScroll, { passive: true });
  syncAmbientMotion();
});

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('scroll', handleScroll);
  window.removeEventListener('resize', updateMotionPrefs);
  if (motionMedia) motionMedia.removeEventListener('change', updateMotionPrefs);
  if (animationFrame) cancelAnimationFrame(animationFrame);
  if (audioPlayer.value) { audioPlayer.value.pause(); audioPlayer.value = null; }
});

function handleMouseMove(e: MouseEvent) {
  requestAnimationFrame(() => { mouseX.value = e.clientX / window.innerWidth; mouseY.value = e.clientY / window.innerHeight; });
}
function handleScroll() { scrollY.value = window.scrollY; }
function updateMotionPrefs() {
  if (!motionMedia) motionMedia = window.matchMedia('(prefers-reduced-motion: reduce)');
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
      const animate = () => { time.value += 0.008; animationFrame = requestAnimationFrame(animate); };
      animate();
    }
  } else {
    window.removeEventListener('mousemove', handleMouseMove);
    if (animationFrame) cancelAnimationFrame(animationFrame);
    isAnimating = false;
  }
}
async function toggleLightAmbient() {
  isLightAmbient.value = !isLightAmbient.value;
  if (isTauri()) await setSetting('gmazz_light_ambient', isLightAmbient.value ? '1' : '0');
  else localStorage.setItem('gmazz_light_ambient', isLightAmbient.value ? '1' : '0');
  syncAmbientMotion();
}
function openNote(note: Note) { router.push({ name: 'note', params: { id: note.id } }); }
function openCreate() { router.push({ name: 'create' }); }
async function openRandomNote() {
  if (!isTauri()) { alert('Случайная запись доступна только в офлайн-режиме.'); return; }
  const note = await store.fetchRandomNote();
  if (note) router.push({ name: 'note', params: { id: note.id } });
}
function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', { day: 'numeric', month: 'short', year: 'numeric' });
}
function truncate(text: string, len: number): string {
  return text.length > len ? text.slice(0, len) + '…' : text;
}
async function togglePlayback(event: Event, note: Note) {
  event.stopPropagation();
  if (currentPlayingId.value === note.id) {
    if (audioPlayer.value) { audioPlayer.value.pause(); audioPlayer.value = null; }
    currentPlayingId.value = null; return;
  }
  if (audioPlayer.value) audioPlayer.value.pause();
  let audioPath = '';
  if (note.note_type === 'phrase' && note.metadata?.file_path) audioPath = note.metadata.file_path;
  else if (note.note_type === 'score' && note.metadata?.audio_path) audioPath = note.metadata.audio_path;
  if (!audioPath) return;
  try {
    const url = await getAssetPath(audioPath);
    const audio = new Audio(url);
    audioPlayer.value = audio;
    currentPlayingId.value = note.id;
    audio.addEventListener('ended', () => { currentPlayingId.value = null; audioPlayer.value = null; });
    await audio.play();
  } catch (e) { console.error('Failed to play audio', e); currentPlayingId.value = null; }
}

const headerScrolled = computed(() => scrollY.value > 40);
const parallaxOffset = computed(() => scrollY.value * 0.3);

const featuredNotes = computed(() => {
  const notes = store.filteredNotes || [];
  if (featuredNoteIds.value.length > 0) {
    const byId = new Map(notes.map(note => [note.id, note]));
    return featuredNoteIds.value.map(id => byId.get(id)).filter(Boolean) as Note[];
  }
  const score = notes.find(n => n.note_type === 'score');
  const phrase = notes.find(n => n.note_type === 'phrase');
  return [score, phrase].filter(Boolean) as Note[];
});

const thoughtHighlights = computed(() => (store.filteredNotes || []).filter(n => n.note_type === 'thought').slice(0, 3));
const selectableNotes = computed(() => store.filteredNotes.slice(0, 30));

function toggleFeatured(noteId: string) {
  const idx = featuredNoteIds.value.indexOf(noteId);
  if (idx >= 0) featuredNoteIds.value.splice(idx, 1);
  else featuredNoteIds.value.push(noteId);
  persistFeaturedNotes();
}

const noteTypeStats = computed(() => {
  const counts: Record<string, number> = {};
  store.notes.forEach(n => { counts[n.note_type] = (counts[n.note_type] || 0) + 1; });
  return counts;
});
</script>

<template>
  <div class="gz-root">

    <!-- ═══ CANVAS BACKGROUND ═══ -->
    <div class="gz-canvas" />
    <div v-if="!isReducedMotion && !isLightAmbient" class="gz-orbs">
      <div class="gz-orb gz-orb--a" :style="{ transform: `translate(${mouseX * 55}px, ${mouseY * 38}px)` }" />
      <div class="gz-orb gz-orb--b" :style="{ transform: `translate(${-mouseX * 42}px, ${-mouseY * 28}px)` }" />
      <div class="gz-orb gz-orb--c" :style="{ transform: `translate(${Math.sin(time) * 44}px, ${Math.cos(time * 0.7) * 32}px)` }" />
    </div>

    <!-- Floating musical glyphs -->
    <div v-if="!isReducedMotion && !isSmallScreen && !isLightAmbient" class="gz-glyphs" aria-hidden="true">
      <span v-for="i in 20" :key="i" class="gz-glyph"
            :style="{
          left: `${(i * 14 + 3) % 100}%`,
          top: `${(i * 19 + 6) % 100}%`,
          fontSize: `${13 + (i % 3) * 8}px`,
          animationDuration: `${10 + i % 8}s`,
          animationDelay: `${i * 0.5}s`
        }">{{ ['𝅝','♩','♪','♫','♬','𝄞','♮','◈'][i % 8] }}</span>
    </div>

    <!-- ═══ HEADER ═══ -->
    <header class="gz-header" :class="{ 'gz-header--solid': headerScrolled }">
      <div class="gz-header__inner">

        <!-- Wordmark -->
        <button class="gz-wordmark" @click="router.push('/')">
          <div class="gz-wordmark__badge">
            <span class="gz-wordmark__clef">𝄞</span>
          </div>
          <div class="gz-wordmark__text">
            <span class="gz-wordmark__name">Gmazz</span>
            <span class="gz-wordmark__sub">Личный архив</span>
          </div>
        </button>

        <!-- Nav right -->
        <nav class="gz-nav">
          <button class="gz-nav__pill" @click="toggleLightAmbient" :aria-pressed="isLightAmbient">
            <span>{{ isLightAmbient ? '☀︎' : '☾' }}</span>
            <span>{{ isLightAmbient ? 'полный фон' : 'лёгкий фон' }}</span>
          </button>
          <button class="gz-nav__pill" @click="openRandomNote">
            <span class="gz-nav__pill-icon">✦</span>
            <span>случайная</span>
          </button>
          <button class="gz-btn-primary" @click="openCreate">
            <span class="gz-btn-primary__plus">+</span>
            новая запись
          </button>
        </nav>

      </div>
    </header>

    <input ref="photoInputRef" type="file" accept="image/*" class="gz-hidden" @change="handlePhotoChange" />

    <!-- ═══════════════════════════════════════════════════ -->
    <!-- HERO -->
    <!-- ═══════════════════════════════════════════════════ -->
    <section class="gz-hero" :style="{ transform: `translateY(${-parallaxOffset * 0.2}px)` }">

      <!-- Five-line stave -->
      <div class="gz-stave" aria-hidden="true">
        <div v-for="i in 5" :key="i" class="gz-stave__line" :style="{ top: `${15 + i * 14}%` }" />
      </div>

      <div class="gz-hero__inner">

        <!-- LEFT: Text column -->
        <div class="gz-hero__text">

          <!-- Eyebrow -->
          <div class="gz-eyebrow">
            <span class="gz-eyebrow__ornament">❧</span>
            <span class="gz-eyebrow__rule" />
            <span class="gz-eyebrow__label">Архив маэстро</span>
          </div>

          <!-- Name -->
          <h1 class="gz-hero__name">
            <span class="gz-hero__name-row">
              <input v-if="canEditSettings" v-model="profileData.firstName"
                     @change="saveProfileField('profile_firstname', profileData.firstName)"
                     class="gz-name-input" :style="{ width: profileData.firstName.length + 'ch' }" aria-label="Имя" />
              <span v-else class="gz-hero__firstname">{{ profileData.firstName }}</span>
              <input v-if="canEditSettings" v-model="profileData.lastName"
                     @change="saveProfileField('profile_lastname', profileData.lastName)"
                     class="gz-name-input gz-name-input--gold" :style="{ width: '6ch' }" aria-label="Фамилия" />
              <span v-else class="gz-hero__lastname">{{ profileData.lastName }}</span>
            </span>
            <input v-if="canEditSettings" v-model="profileData.role"
                   @change="saveProfileField('profile_role', profileData.role)"
                   class="gz-name-input gz-name-input--role" aria-label="Роль" />
            <span v-else class="gz-hero__role">{{ profileData.role }}</span>
          </h1>

          <!-- Credential badges -->
          <div class="gz-badges">
            <div class="gz-badge gz-badge--violet">
              <div class="gz-badge__shimmer" />
              <div class="gz-badge__body">
                <div class="gz-badge__icon">
                  <span>𝄞</span>
                </div>
                <div class="gz-badge__copy">
                  <div class="gz-badge__title">Композитор и аранжировщик</div>
                  <div class="gz-badge__subtitle">Мультиинструменталист</div>
                </div>
              </div>
            </div>
            <div class="gz-badge gz-badge--amber">
              <div class="gz-badge__shimmer" />
              <div class="gz-badge__body">
                <div class="gz-badge__icon">
                  <span>🎹</span>
                </div>
                <div class="gz-badge__copy">
                  <div class="gz-badge__title">Профессиональный преподаватель</div>
                  <div class="gz-badge__subtitle">Игра на фортепиано</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Description -->
          <div class="gz-hero__desc-wrap">
            <div class="gz-hero__desc-bar" />
            <textarea v-if="canEditSettings" v-model="profileData.description"
                      @change="saveProfileField('profile_description', profileData.description)"
                      rows="2" class="gz-hero__desc-edit" />
            <p v-else class="gz-hero__desc">{{ profileData.description }}</p>
          </div>

          <!-- Stats row -->
          <div class="gz-stats">
            <div class="gz-stat">
              <span class="gz-stat__num">{{ store.notes.length || '—' }}</span>
              <span class="gz-stat__lbl">записей</span>
            </div>
            <div class="gz-stat__div" />
            <button class="gz-stat gz-stat--cta" @click="openCreate">
              <span class="gz-stat__num gz-stat__num--gold">+</span>
              <span class="gz-stat__lbl">создать</span>
            </button>
          </div>

          <!-- Type pills -->
          <div class="gz-type-pills">
            <div v-for="(cfg, type) in typeConfig" :key="type" class="gz-type-pill"
                 :style="{ '--pill-bg': cfg.tag, '--pill-color': cfg.dark, '--pill-border': cfg.accent }">
              <span class="gz-type-pill__icon">{{ cfg.icon }}</span>
              <span class="gz-type-pill__name">{{ cfg.name }}</span>
              <span class="gz-type-pill__count">{{ noteTypeStats[type] || 0 }}</span>
            </div>
          </div>

        </div>

        <!-- RIGHT: Photo -->
        <div class="gz-hero__photo-col">
          <div class="gz-photo" @mouseenter="isPhotoHovered = true" @mouseleave="isPhotoHovered = false">

            <!-- Ambient halo -->
            <div class="gz-photo__halo" :class="{ 'gz-photo__halo--bright': isPhotoHovered }" />

            <!-- Decorative ring -->
            <div class="gz-photo__ring" />

            <!-- Frame -->
            <div class="gz-photo__frame" @click="canEditSettings ? triggerPhotoUpload() : null">
              <template v-if="profilePhotoUrl">
                <img :src="profilePhotoUrl" alt="Профиль" class="gz-photo__img" :class="{ 'gz-photo__img--hover': isPhotoHovered }" />
                <div class="gz-photo__vignette" />
              </template>
              <template v-else>
                <div class="gz-photo__placeholder">
                  <span class="gz-photo__placeholder-clef">𝄞</span>
                  <span class="gz-photo__placeholder-label">Фото</span>
                </div>
              </template>

              <!-- Corner brackets -->
              <div class="gz-photo__corner gz-photo__corner--tl" />
              <div class="gz-photo__corner gz-photo__corner--tr" />
              <div class="gz-photo__corner gz-photo__corner--bl" />
              <div class="gz-photo__corner gz-photo__corner--br" />

              <!-- Upload overlay -->
              <div v-if="canEditSettings" class="gz-photo__overlay" :class="{ 'gz-photo__overlay--show': isPhotoHovered }">
                <div class="gz-photo__upload-btn">
                  <span v-if="isUploadingPhoto" class="gz-spin">⟳</span>
                  <span v-else>📷</span>
                  <span>{{ isUploadingPhoto ? 'Загрузка...' : (profilePhotoUrl ? 'Изменить' : 'Добавить') }}</span>
                </div>
              </div>
            </div>

            <!-- Floating notes -->
            <span class="gz-photo__float gz-photo__float--a">♪</span>
            <span class="gz-photo__float gz-photo__float--b">♫</span>
            <span class="gz-photo__float gz-photo__float--c">𝅘𝅥𝅮</span>
          </div>
        </div>

      </div>
    </section>

    <!-- ═══════════════════════════════════════════════════ -->
    <!-- GENRES -->
    <!-- ═══════════════════════════════════════════════════ -->
    <section class="gz-section gz-section--genres">
      <div class="gz-container">

        <!-- Section header -->
        <div class="gz-section-head">
          <div class="gz-section-head__ornaments">
            <div class="gz-section-head__rule gz-section-head__rule--left" />
            <span class="gz-section-head__floret">❧</span>
            <div class="gz-section-head__rule gz-section-head__rule--right" />
          </div>
          <h2 class="gz-section-head__title">Музыкальные жанры</h2>
          <p class="gz-section-head__sub">Работал в таких жанрах как джаз, госпел, акапелла и классика</p>
        </div>

        <!-- Genre grid -->
        <div class="gz-genres">
          <div v-for="(genre, idx) in genres" :key="genre.name"
               class="gz-genre"
               :style="{
                 '--g-color': genre.color,
                 '--g-glow': genre.glow,
                 animationDelay: `${idx * 0.12}s`
               }">

            <!-- Background -->
            <div class="gz-genre__bg" :style="{ background: genre.gradient }" />

            <!-- Floating symbols -->
            <div class="gz-genre__symbols" aria-hidden="true">
              <span v-for="k in 9" :key="k" class="gz-genre__sym"
                    :style="{ left: `${(k * 24) % 100}%`, top: `${(k * 18) % 100}%`, animationDelay: `${k * 0.35}s` }">
                {{ genre.pattern.split(' ')[k % 3] }}
              </span>
            </div>

            <!-- Card body -->
            <div class="gz-genre__body">
              <div class="gz-genre__top">
                <div class="gz-genre__icon-wrap">
                  <div class="gz-genre__icon-glow" />
                  <span class="gz-genre__icon">{{ genre.icon }}</span>
                </div>
                <span class="gz-genre__idx">{{ String(idx + 1).padStart(2, '0') }}</span>
              </div>

              <div class="gz-genre__info">
                <h3 class="gz-genre__name">{{ genre.name }}</h3>
                <div class="gz-genre__sub">{{ genre.sub }}</div>
                <p class="gz-genre__desc">{{ genre.desc }}</p>
              </div>

              <!-- Mini stave -->
              <div class="gz-genre__stave">
                <div v-for="l in 5" :key="l" class="gz-genre__stave-line" />
              </div>

              <div class="gz-genre__arrow">→</div>
            </div>

            <!-- Hover bloom -->
            <div class="gz-genre__bloom" />
          </div>
        </div>

      </div>
    </section>

    <!-- ═══════════════════════════════════════════════════ -->
    <!-- MAESTRO ARCHIVE -->
    <!-- ═══════════════════════════════════════════════════ -->
    <section class="gz-section">
      <div class="gz-container">

        <!-- Divider -->
        <div class="gz-divider">
          <span class="gz-divider__ornament">❧</span>
          <div class="gz-divider__line" />
          <span class="gz-divider__label">Архив</span>
        </div>

        <div class="gz-archive-grid">

          <!-- ── INFO PANEL ── -->
          <div class="gz-archive-info">
            <div class="gz-archive-info__inner">

              <div class="gz-archive-info__header">
                <div class="gz-archive-info__label">
                  <span class="gz-archive-info__clef">𝄞</span>
                  <input v-if="canEditSettings" v-model="maestroBlock.title"
                         @change="saveMaestroField('maestro_title', maestroBlock.title)"
                         class="gz-inline-edit gz-inline-edit--label" />
                  <span v-else class="gz-archive-info__title-text">{{ maestroBlock.title }}</span>
                </div>
                <button v-if="canEditSettings" class="gz-edit-btn" @click="isEditingFeatured = !isEditingFeatured">
                  {{ isEditingFeatured ? '✓ готово' : '✎ ред.' }}
                </button>
              </div>

              <div v-if="canEditSettings" contenteditable="true" class="gz-archive-headline-edit"
                   @blur="saveMaestroField('maestro_headline', ($event.target as HTMLElement).innerText)">{{ maestroBlock.headline }}</div>
              <h2 v-else class="gz-archive-headline">{{ maestroBlock.headline }}</h2>

              <div v-if="canEditSettings" contenteditable="true" class="gz-archive-desc-edit"
                   @blur="saveMaestroField('maestro_description', ($event.target as HTMLElement).innerText)">{{ maestroBlock.description }}</div>
              <p v-else class="gz-archive-desc">{{ maestroBlock.description }}</p>

              <!-- CTA -->
              <div class="gz-archive-cta-row">
                <button class="gz-btn-primary" @click="openCreate">
                  <input v-if="canEditSettings" v-model="maestroBlock.cta"
                         @change="saveMaestroField('maestro_cta', maestroBlock.cta)"
                         @click.stop class="gz-inline-edit gz-inline-edit--cta" />
                  <span v-else>{{ maestroBlock.cta }}</span>
                </button>
                <span class="gz-archive-years">
                  {{ profileData.startYear }} — {{ new Date().getFullYear() }}
                </span>
              </div>

              <!-- Mini stats -->
              <div class="gz-mini-stats">
                <div class="gz-mini-stat">
                  <span class="gz-mini-stat__val">{{ store.notes.length || '—' }}</span>
                  <span class="gz-mini-stat__lbl">записей</span>
                </div>
                <div class="gz-mini-stat__sep" />
                <div class="gz-mini-stat">
                  <span class="gz-mini-stat__val">4</span>
                  <span class="gz-mini-stat__lbl">жанра</span>
                </div>
                <div class="gz-mini-stat__sep" />
                <div class="gz-mini-stat gz-mini-stat--gold">
                  <span class="gz-mini-stat__val">∞</span>
                  <span class="gz-mini-stat__lbl">мотивов</span>
                </div>
              </div>

            </div>
          </div>

          <!-- ── FEATURED + THOUGHTS ── -->
          <div class="gz-archive-right">

            <!-- Featured notes -->
            <div class="gz-featured">
              <div class="gz-featured__header">
                <h3 class="gz-featured__title">Избранные произведения</h3>
                <span class="gz-featured__all">архив →</span>
              </div>

              <div v-if="featuredNotes.length" class="gz-featured__grid">
                <button v-for="note in featuredNotes" :key="note.id"
                        class="gz-feat-note" @click="openNote(note)">
                  <div class="gz-feat-note__type">
                    <span class="gz-feat-note__icon"
                          :style="{ background: typeConfig[note.note_type].tag, color: typeConfig[note.note_type].dark }">
                      {{ typeConfig[note.note_type].icon }}
                    </span>
                    <span class="gz-feat-note__kind" :style="{ color: typeConfig[note.note_type].accent }">
                      {{ typeConfig[note.note_type].name }}
                    </span>
                  </div>
                  <p class="gz-feat-note__text">{{ note.content || 'Без описания' }}</p>
                  <span class="gz-feat-note__date">{{ formatDate(note.created_at) }}</span>
                </button>
              </div>

              <div v-else class="gz-featured__empty">
                <span class="gz-featured__empty-clef">𝄞</span>
                <p>Избранных произведений пока нет</p>
                <p v-if="canEditSettings" class="gz-featured__empty-hint">Нажмите «ред.» чтобы выбрать</p>
              </div>

              <!-- Edit panel -->
              <div v-if="canEditSettings && isEditingFeatured" class="gz-select-panel">
                <div class="gz-select-panel__title">Выбрать избранные</div>
                <div class="gz-select-grid">
                  <label v-for="note in selectableNotes" :key="note.id" class="gz-select-item">
                    <input type="checkbox" class="gz-select-item__check"
                           :checked="featuredNoteIds.includes(note.id)" @change="toggleFeatured(note.id)" />
                    <span :style="{ color: typeConfig[note.note_type].accent }">{{ typeConfig[note.note_type].icon }}</span>
                    <span class="gz-select-item__text">{{ note.content || 'Без описания' }}</span>
                  </label>
                </div>
              </div>
            </div>

            <!-- Thought highlights -->
            <div v-if="thoughtHighlights.length" class="gz-thoughts">
              <div class="gz-thoughts__header">
                <span class="gz-thoughts__ornament">❧</span>
                <h3 class="gz-thoughts__title">Мысли мастера</h3>
              </div>
              <div class="gz-thoughts__grid">
                <div v-for="note in thoughtHighlights" :key="note.id" class="gz-thought">
                  <div class="gz-thought__quote-mark">"</div>
                  <p class="gz-thought__text">{{ note.content }}</p>
                </div>
              </div>
            </div>

          </div>
        </div>
      </div>
    </section>

    <!-- ═══════════════════════════════════════════════════ -->
    <!-- NOTES COLLECTION -->
    <!-- ═══════════════════════════════════════════════════ -->
    <section class="gz-section gz-section--collection">
      <div class="gz-container">

        <div class="gz-collection-head">
          <div>
            <div class="gz-collection-head__eyebrow">Коллекция работ</div>
            <div class="gz-collection-head__accent-bar" />
          </div>
          <div class="gz-collection-head__rule" />
          <span class="gz-collection-head__count">{{ store.filteredNotes.length }} записей</span>
        </div>

        <!-- Grid -->
        <div v-if="store.filteredNotes.length > 0" class="gz-notes-grid">
          <NoteCard
              v-for="(note, index) in store.filteredNotes"
              :key="note.id"
              :note="note"
              :index="index"
              :current-playing-id="currentPlayingId"
              @open="openNote"
              @toggle-playback="togglePlayback"
          />
        </div>

        <!-- Empty state -->
        <div v-else-if="!store.loading" class="gz-empty">
          <div class="gz-empty__card">
            <span class="gz-empty__clef" :style="{ transform: `scale(${1 + Math.sin(time)*0.06})`, display:'inline-block' }">𝄞</span>
            <h3 class="gz-empty__title">Архив пуст</h3>
            <p class="gz-empty__body">Создайте первую запись, чтобы начать формировать коллекцию музыкальных идей</p>
            <button class="gz-btn-primary gz-btn-primary--lg" @click="openCreate">+ создать запись</button>
          </div>
        </div>

        <!-- Loading -->
        <div v-if="store.loading" class="gz-loading">
          <div class="gz-loading__ring">
            <div class="gz-loading__ping" />
            <div class="gz-loading__spin" />
          </div>
          <span class="gz-loading__label">загрузка...</span>
        </div>

        <!-- Load more -->
        <div v-if="store.hasMore && store.filteredNotes.length > 0" class="gz-load-more">
          <button class="gz-btn-outline" @click="store.loadMore()" :disabled="store.loading">
            показать ещё
          </button>
        </div>

      </div>
    </section>

    <!-- ═══ FOOTER ═══ -->
    <footer class="gz-footer">
      <div class="gz-footer__inner">
        <div class="gz-footer__brand">
          <span class="gz-footer__clef" :style="{ transform: `scale(${1 + Math.sin(time*0.4)*0.05})`, display:'inline-block' }">𝄞</span>
          <div>
            <div class="gz-footer__name">Gmazz</div>
            <div class="gz-footer__years">
              архив
              <input v-if="canEditSettings" v-model="profileData.startYear"
                     @change="saveProfileField('archive_start_year', profileData.startYear)"
                     class="gz-inline-edit gz-inline-edit--footer-year" />
              <span v-else>{{ profileData.startYear }}</span>
              — {{ new Date().getFullYear() }}
            </div>
          </div>
        </div>
        <div class="gz-footer__quote-wrap">
          <div v-if="canEditSettings" contenteditable="true" class="gz-footer__quote gz-footer__quote--edit"
               @blur="saveProfileField('profile_quote', ($event.target as HTMLElement).innerText)">{{ profileData.quote }}</div>
          <p v-else class="gz-footer__quote">{{ profileData.quote }}</p>
        </div>
      </div>
    </footer>

  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300;1,400;1,600&family=Playfair+Display:ital,wght@0,700;0,800;0,900;1,400&family=DM+Sans:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');

/* ═══════════════════ CSS VARIABLES ═══════════════════ */
:root, .gz-root {
  --amber-50:  #FFFBF0;
  --amber-100: #FEF3D0;
  --amber-200: #FBDFA0;
  --amber-300: #F5C460;
  --amber-400: #EBA832;
  --amber-500: #D4881A;
  --amber-600: #B8690C;
  --amber-700: #964E07;
  --amber-800: #763804;
  --amber-900: #5A2803;

  --stone-50:  #FDFAF4;
  --stone-100: #F7F1E6;
  --stone-200: #EDE4D2;
  --stone-300: #D6C8AE;
  --stone-400: #B8A48A;
  --stone-500: #9A8068;
  --stone-600: #7C5E48;
  --stone-700: #5E4030;
  --stone-800: #3E2618;
  --stone-900: #1E1008;

  --ink:       #1A0E06;
  --card-bg:   rgba(255, 255, 255, 0.82);
  --card-border: rgba(210, 185, 145, 0.42);

  --radius-sm:  12px;
  --radius-md:  20px;
  --radius-lg:  28px;
  --radius-xl:  36px;

  --shadow-sm:  0 2px 8px rgba(100, 60, 10, 0.06);
  --shadow-md:  0 8px 32px rgba(100, 60, 10, 0.10);
  --shadow-lg:  0 20px 60px rgba(100, 60, 10, 0.14);
  --shadow-xl:  0 32px 80px rgba(100, 60, 10, 0.18);
}

/* ═══════════════════ BASE ═══════════════════ */
* { -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; }
html { scroll-behavior: smooth; }

.gz-root {
  min-height: 100vh;
  overflow-x: hidden;
  font-family: 'DM Sans', system-ui, sans-serif;
  color: var(--ink);
}

.gz-hidden { display: none; }

/* ═══════════════════ CANVAS BACKGROUND ═══════════════════ */
.gz-canvas {
  position: fixed; inset: 0; z-index: -20;
  background:
      radial-gradient(ellipse 140% 90% at 10% 0%,   rgba(255, 222, 140, 0.36) 0%, transparent 55%),
      radial-gradient(ellipse 100% 80% at 90% 100%,  rgba(228, 198, 155, 0.28) 0%, transparent 52%),
      radial-gradient(ellipse 70%  60% at 50% 50%,   rgba(255, 244, 210, 0.18) 0%, transparent 60%),
      linear-gradient(170deg, #FDFAF4 0%, #FAF3E2 30%, #F4EAD5 65%, #EDE0C8 100%);
}

/* Subtle paper grain overlay */
.gz-canvas::after {
  content: '';
  position: absolute; inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='0.025'/%3E%3C/svg%3E");
  background-size: 200px 200px;
  opacity: 0.6;
  pointer-events: none;
}

/* Ambient orbs */
.gz-orbs { position: fixed; inset: 0; z-index: -15; pointer-events: none; overflow: hidden; }
.gz-orb { position: absolute; border-radius: 50%; filter: blur(90px); transition: transform 0.15s ease-out; }
.gz-orb--a { width: 900px; height: 900px; top: -20%; left: -12%; background: radial-gradient(circle, rgba(255, 205, 100, 0.28) 0%, rgba(240, 168, 72, 0.10) 45%, transparent 72%); }
.gz-orb--b { width: 680px; height: 680px; bottom: -18%; right: -10%; background: radial-gradient(circle, rgba(210, 172, 120, 0.22) 0%, rgba(188, 148, 100, 0.08) 45%, transparent 72%); }
.gz-orb--c { width: 520px; height: 520px; top: 40%; left: 42%; background: radial-gradient(circle, rgba(252, 230, 180, 0.20) 0%, transparent 65%); }

/* Floating glyphs */
.gz-glyphs { position: fixed; inset: 0; z-index: -10; pointer-events: none; overflow: hidden; }
.gz-glyph {
  position: absolute; font-family: 'Cormorant Garamond', serif; user-select: none;
  color: rgba(168, 110, 38, 0.055);
  animation: gz-drift linear infinite;
}
@keyframes gz-drift {
  0%   { transform: translate(0, 0) rotate(0deg); opacity: 0.4; }
  30%  { transform: translate(14px, -22px) rotate(80deg); opacity: 0.75; }
  65%  { transform: translate(-9px, -14px) rotate(175deg); opacity: 0.55; }
  100% { transform: translate(0, 0) rotate(360deg); opacity: 0.4; }
}

/* ═══════════════════ HEADER ═══════════════════ */
.gz-header {
  position: fixed; top: 0; left: 0; right: 0; z-index: 100;
  transition: all 0.5s cubic-bezier(0.22, 1, 0.36, 1);
}
.gz-header--solid {
  background: rgba(252, 248, 242, 0.88);
  backdrop-filter: blur(28px) saturate(1.4);
  -webkit-backdrop-filter: blur(28px) saturate(1.4);
  border-bottom: 1px solid rgba(200, 168, 110, 0.18);
  box-shadow: 0 1px 0 rgba(200, 168, 110, 0.12), 0 6px 32px rgba(90, 50, 10, 0.07);
}
.gz-header__inner {
  max-width: 1280px; margin: 0 auto;
  padding: 0 48px; height: 76px;
  display: flex; align-items: center; justify-content: space-between;
}

/* Wordmark */
.gz-wordmark { display: flex; align-items: center; gap: 14px; border: none; background: none; cursor: pointer; }
.gz-wordmark__badge {
  width: 44px; height: 44px; border-radius: 14px;
  background: linear-gradient(145deg, #FEF3DC 0%, #FDE1A0 60%, #F9C868 100%);
  border: 1.5px solid rgba(196, 148, 58, 0.3);
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 4px 14px rgba(148, 88, 10, 0.20), inset 0 1px 0 rgba(255, 255, 255, 0.7);
  transition: transform 0.5s cubic-bezier(0.22, 1, 0.36, 1), box-shadow 0.3s;
}
.gz-wordmark:hover .gz-wordmark__badge { transform: rotate(14deg); box-shadow: 0 8px 22px rgba(148, 88, 10, 0.28); }
.gz-wordmark__clef { font-family: 'Cormorant Garamond', serif; font-size: 22px; color: #9A5C10; line-height: 1; }
.gz-wordmark__text { display: flex; flex-direction: column; gap: 1px; }
.gz-wordmark__name {
  font-family: 'Cormorant Garamond', serif; font-size: 22px; font-weight: 700; line-height: 1;
  color: var(--stone-800); letter-spacing: -0.01em;
  transition: color 0.3s;
}
.gz-wordmark:hover .gz-wordmark__name { color: var(--amber-600); }
.gz-wordmark__sub { font-size: 9px; text-transform: uppercase; letter-spacing: 0.42em; color: var(--stone-400); font-weight: 600; }

/* Nav */
.gz-nav { display: flex; align-items: center; gap: 8px; }
.gz-nav__pill {
  display: flex; align-items: center; gap: 6px;
  padding: 9px 16px; border-radius: 12px; border: none; background: none; cursor: pointer;
  font-family: 'DM Sans', sans-serif; font-size: 13px; color: var(--stone-600);
  transition: background 0.22s, color 0.22s;
}
.gz-nav__pill:hover { background: rgba(248, 230, 185, 0.58); color: var(--amber-700); }
.gz-nav__pill-icon { color: var(--amber-500); font-size: 13px; }

/* Primary CTA button */
.gz-btn-primary {
  display: inline-flex; align-items: center; gap: 7px;
  padding: 11px 22px; border-radius: 14px; border: none; cursor: pointer;
  background: linear-gradient(150deg, #C98238 0%, #A86018 55%, #8A4808 100%);
  color: white; font-family: 'DM Sans', sans-serif; font-size: 13px; font-weight: 600;
  letter-spacing: 0.01em;
  box-shadow: 0 4px 18px rgba(152, 88, 14, 0.34), 0 1px 0 rgba(255, 255, 255, 0.12) inset, 0 -1px 0 rgba(0,0,0,0.15) inset;
  transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
  position: relative; overflow: hidden;
}
.gz-btn-primary::after {
  content: '';
  position: absolute; top: 0; left: -100%; width: 40%; height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.16), transparent);
  transition: left 0.55s;
}
.gz-btn-primary:hover::after { left: 160%; }
.gz-btn-primary:hover { transform: translateY(-2px); box-shadow: 0 10px 28px rgba(152, 88, 14, 0.44), 0 1px 0 rgba(255, 255, 255, 0.16) inset; }
.gz-btn-primary:active { transform: translateY(0); }
.gz-btn-primary--lg { padding: 15px 42px; font-size: 15px; border-radius: 16px; }
.gz-btn-primary__plus { font-size: 18px; line-height: 1; font-weight: 300; }

/* Outline button */
.gz-btn-outline {
  padding: 14px 52px; border-radius: 16px;
  border: 1.5px solid var(--stone-300);
  background: rgba(253, 250, 244, 0.85); backdrop-filter: blur(10px);
  font-family: 'DM Sans', sans-serif; font-size: 14px; font-weight: 600; color: var(--stone-600);
  cursor: pointer; transition: all 0.25s;
}
.gz-btn-outline:hover { border-color: var(--amber-500); color: var(--amber-700); background: rgba(255, 244, 220, 0.9); transform: translateY(-2px); box-shadow: 0 8px 24px rgba(152, 88, 14, 0.10); }
.gz-btn-outline:disabled { opacity: 0.4; cursor: not-allowed; }

/* ═══════════════════ LAYOUT HELPERS ═══════════════════ */
.gz-container { max-width: 1280px; margin: 0 auto; padding: 0 48px; }
.gz-section { padding: 0 0 100px; }
.gz-section--genres { padding: 0 0 108px; }
.gz-section--collection { padding: 0 0 160px; }

/* ═══════════════════ HERO ═══════════════════ */
.gz-hero {
  position: relative;
  padding: 168px 48px 100px;
  max-width: 1280px; margin: 0 auto;
  overflow: hidden;
}

/* Five-line stave */
.gz-stave { position: absolute; inset: 0; pointer-events: none; }
.gz-stave__line { position: absolute; left: 0; right: 0; height: 1px; background: rgba(162, 112, 38, 0.042); }

.gz-hero__inner {
  display: grid;
  grid-template-columns: 1fr 330px;
  gap: 80px; align-items: center;
}
@media (max-width: 1024px) { .gz-hero__inner { grid-template-columns: 1fr; } .gz-hero__photo-col { display: none; } }

/* Eyebrow */
.gz-eyebrow {
  display: flex; align-items: center; gap: 14px; margin-bottom: 36px;
}
.gz-eyebrow__ornament { font-size: 24px; color: var(--amber-500); font-family: 'Cormorant Garamond', serif; line-height: 1; }
.gz-eyebrow__rule { width: 44px; height: 1px; background: linear-gradient(90deg, var(--amber-400), var(--amber-200)); }
.gz-eyebrow__label { font-size: 10px; text-transform: uppercase; letter-spacing: 0.48em; color: rgba(168, 100, 28, 0.65); font-weight: 700; }

/* Name heading */
.gz-hero__name {
  font-family: 'Cormorant Garamond', serif;
  line-height: 1.02; letter-spacing: -0.035em;
  margin-bottom: 0;
}
.gz-hero__name-row { display: flex; flex-wrap: wrap; align-items: center; gap: 0 20px; }
.gz-hero__firstname {
  font-size: clamp(50px, 10vw, 130px); font-weight: 600; color: var(--stone-900);
}
.gz-hero__lastname {
  font-size: clamp(50px, 10vw, 120px); font-weight: 600;
  background: linear-gradient(135deg, #C98238 0%, #8A4808 100%);
  -webkit-background-clip: text; -webkit-text-fill-color: transparent;
  background-clip: text;
}
.gz-hero__role {
  display: block; font-size: clamp(40px, 10vw, 100px); font-weight: 500;
  color: var(--stone-600); letter-spacing: 0.01em; line-height: 1.3; margin-top: 6px;
  font-style: italic;
}

/* Name inputs */
.gz-name-input {
  background: transparent; border: none; border-bottom: 2px solid transparent;
  outline: none; font: inherit; color: inherit; letter-spacing: inherit;
  transition: border-color 0.22s;
}
.gz-hero__name-row .gz-name-input {
  font-size: clamp(40px, 10vw, 120px);
  font-weight: 600;
}
.gz-name-input:hover { border-bottom-color: rgba(190, 120, 40, 0.4); }
.gz-name-input:focus { border-bottom-color: var(--amber-500); }
.gz-name-input--gold { background: linear-gradient(135deg, #C98238 0%, #8A4808 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
.gz-name-input--role { color: var(--stone-400); font-style: italic; width: 100%; font-size: clamp(20px, 6vw, 120px); }

/* Credential badges */
.gz-badges { display: flex; flex-direction: column; gap: 12px; margin-top: 28px; }
.gz-badge {
  position: relative; overflow: hidden; border-radius: 20px;
  transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1), box-shadow 0.4s;
  cursor: default;
}
.gz-badge::before {
  content: ''; position: absolute; inset: 0; border-radius: 20px;
  padding: 1.5px;
  background: linear-gradient(135deg, rgba(255,255,255,0.55), rgba(255,255,255,0.08), rgba(255,255,255,0.32));
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor; mask-composite: exclude; pointer-events: none;
}
.gz-badge:hover { transform: translateY(-3px) scale(1.01); }

.gz-badge--amber {
  background: linear-gradient(140deg, #FEF5E0 0%, #FDEAC5 50%, #FBE0A2 100%);
  box-shadow: 0 4px 18px rgba(192, 120, 50, 0.16), inset 0 1px 0 rgba(255,255,255,0.8);
}
.gz-badge--amber:hover { box-shadow: 0 14px 36px rgba(192, 120, 50, 0.26), inset 0 1px 0 rgba(255,255,255,0.88); }

.gz-badge--violet {
  background: linear-gradient(140deg, #F4EEFF 0%, #EADDFB 50%, #DCC8F5 100%);
  box-shadow: 0 4px 18px rgba(120, 72, 162, 0.15), inset 0 1px 0 rgba(255,255,255,0.8);
}
.gz-badge--violet:hover { box-shadow: 0 14px 36px rgba(120, 72, 162, 0.24), inset 0 1px 0 rgba(255,255,255,0.88); }

.gz-badge__shimmer {
  position: absolute; top: 0; left: -100%; width: 44%; height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.32), transparent);
  animation: gz-shimmer 3.5s ease-in-out infinite;
}
@keyframes gz-shimmer { 0%, 100% { left: -100%; } 50% { left: 150%; } }

.gz-badge__body { position: relative; z-index: 1; display: flex; align-items: center; gap: 16px; padding: 18px 22px; }
.gz-badge__icon {
  width: 54px; height: 54px; border-radius: 15px; display: flex; align-items: center; justify-content: center;
  font-family: 'Cormorant Garamond', serif; font-size: 24px; flex-shrink: 0;
  transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1);
}
.gz-badge--amber .gz-badge__icon { background: linear-gradient(140deg, rgba(255,255,255,0.92), rgba(255,244,215,0.78)); box-shadow: 0 3px 10px rgba(192,120,50,0.18), inset 0 1px 0 rgba(255,255,255,0.95); }
.gz-badge--violet .gz-badge__icon { background: linear-gradient(140deg, rgba(255,255,255,0.92), rgba(242,232,255,0.78)); box-shadow: 0 3px 10px rgba(120,72,162,0.18), inset 0 1px 0 rgba(255,255,255,0.95); }
.gz-badge:hover .gz-badge__icon { transform: scale(1.1) rotate(-5deg); }

.gz-badge__copy { flex: 1; min-width: 0; }
.gz-badge__title { font-family: 'DM Sans', sans-serif; font-size: 15px; font-weight: 700; line-height: 1.3; margin-bottom: 3px; }
.gz-badge--amber .gz-badge__title { color: #7A4E18; }
.gz-badge--violet .gz-badge__title { color: #4A2E78; }
.gz-badge__subtitle { font-size: 12px; font-weight: 500; opacity: 0.68; }
.gz-badge--amber .gz-badge__subtitle { color: #9A6530; }
.gz-badge--violet .gz-badge__subtitle { color: #6A4A98; }

/* Hero description */
.gz-hero__desc-wrap { margin-top: 36px; padding-left: 20px; border-left: 3px solid rgba(196, 148, 58, 0.48); max-width: 580px; position: relative; }
.gz-hero__desc-bar { display: none; }
.gz-hero__desc { font-family: 'Cormorant Garamond', serif; font-size: 19px; color: var(--stone-500); line-height: 1.75; font-style: italic; }
.gz-hero__desc-edit {
  width: 100%; background: transparent; resize: none; outline: none; border: none;
  font-family: 'Cormorant Garamond', serif; font-size: 19px; color: var(--stone-500);
  line-height: 1.75; font-style: italic;
}

/* Stats */
.gz-stats { display: flex; align-items: stretch; gap: 0; margin-top: 36px; width: fit-content; }
.gz-stat {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 16px 24px; border-radius: 18px; min-width: 88px;
  background: rgba(255, 255, 255, 0.52); border: 1.5px solid rgba(210, 182, 136, 0.36);
  backdrop-filter: blur(12px); transition: all 0.3s cubic-bezier(0.22, 1, 0.36, 1);
}
.gz-stat:hover { background: rgba(255, 252, 244, 0.9); transform: translateY(-4px); box-shadow: 0 12px 32px rgba(148, 88, 14, 0.12); }
.gz-stat--cta { cursor: pointer; }
.gz-stat__num { font-family: 'Cormorant Garamond', serif; font-size: 42px; font-weight: 700; color: var(--ink); line-height: 1; }
.gz-stat__num--gold { color: var(--amber-600); }
.gz-stat__lbl { font-size: 9px; text-transform: uppercase; letter-spacing: 0.3em; color: var(--stone-400); font-weight: 700; margin-top: 4px; }
.gz-stat__div { width: 1px; align-self: stretch; margin: 0 4px; background: linear-gradient(180deg, transparent, rgba(200, 168, 110, 0.38), transparent); }

/* Type pills */
.gz-type-pills { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 32px; }
.gz-type-pill {
  display: inline-flex; align-items: center; gap: 7px;
  padding: 7px 14px; border-radius: 100px;
  background: var(--pill-bg); color: var(--pill-color);
  border: 1px solid color-mix(in srgb, var(--pill-border) 25%, transparent);
  font-size: 12px; font-weight: 600; letter-spacing: 0.01em;
  transition: transform 0.22s, box-shadow 0.22s;
}
.gz-type-pill:hover { transform: translateY(-2px); box-shadow: 0 6px 18px rgba(0,0,0,0.10); }
.gz-type-pill__icon { font-family: 'Cormorant Garamond', serif; font-size: 16px; }
.gz-type-pill__name { font-family: 'DM Sans', sans-serif; font-weight: 600; }
.gz-type-pill__count { font-weight: 700; opacity: 0.55; font-size: 11px; }

/* ═══════════════════ PHOTO ═══════════════════ */
.gz-hero__photo-col { display: flex; justify-content: center; }
.gz-photo { position: relative; display: inline-block; }

.gz-photo__halo {
  position: absolute; inset: -30px; border-radius: 55px;
  background: radial-gradient(ellipse, rgba(255, 205, 100, 0.46) 0%, rgba(240, 175, 82, 0.18) 52%, transparent 76%);
  filter: blur(32px); opacity: 0.55; transition: opacity 0.45s; pointer-events: none;
}
.gz-photo__halo--bright { opacity: 0.95; }

.gz-photo__ring {
  position: absolute; inset: -8px; border-radius: 40px;
  border: 1px solid rgba(196, 148, 58, 0.22);
  background: linear-gradient(135deg, rgba(255,244,220,0.18) 0%, transparent 50%, rgba(255,220,140,0.12) 100%);
  pointer-events: none;
}

.gz-photo__frame {
  position: relative; width: 300px; height: 360px; border-radius: 32px; overflow: hidden;
  background: linear-gradient(145deg, #FEF5E0 0%, #FDE6B0 100%);
  box-shadow: 0 32px 80px rgba(80, 44, 0, 0.22), 0 10px 28px rgba(80, 44, 0, 0.12), inset 0 1px 0 rgba(255,255,255,0.5);
  cursor: pointer; transition: box-shadow 0.45s;
}
.gz-photo__frame:hover { box-shadow: 0 40px 100px rgba(80, 44, 0, 0.28), 0 12px 36px rgba(80, 44, 0, 0.15); }

.gz-photo__img { width: 100%; height: 100%; object-fit: cover; transition: transform 1s cubic-bezier(0.22, 1, 0.36, 1); }
.gz-photo__img--hover { transform: scale(1.06); }
.gz-photo__vignette { position: absolute; inset: 0; background: linear-gradient(to top, rgba(52, 24, 0, 0.18) 0%, transparent 55%); pointer-events: none; }

.gz-photo__placeholder { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: var(--stone-300); }
.gz-photo__placeholder-clef { font-family: 'Cormorant Garamond', serif; font-size: 90px; line-height: 1; margin-bottom: 8px; }
.gz-photo__placeholder-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.32em; font-weight: 600; }

.gz-photo__corner { position: absolute; width: 22px; height: 22px; border-color: rgba(205, 158, 58, 0.55); border-style: solid; pointer-events: none; }
.gz-photo__corner--tl { top: 12px; left: 12px; border-width: 2px 0 0 2px; border-radius: 6px 0 0 0; }
.gz-photo__corner--tr { top: 12px; right: 12px; border-width: 2px 2px 0 0; border-radius: 0 6px 0 0; }
.gz-photo__corner--bl { bottom: 12px; left: 12px; border-width: 0 0 2px 2px; border-radius: 0 0 0 6px; }
.gz-photo__corner--br { bottom: 12px; right: 12px; border-width: 0 2px 2px 0; border-radius: 0 0 6px 0; }

.gz-photo__overlay {
  position: absolute; inset: 0; background: rgba(22, 8, 0, 0.46); backdrop-filter: blur(4px);
  display: flex; align-items: center; justify-content: center; opacity: 0; transition: opacity 0.38s;
}
.gz-photo__overlay--show { opacity: 1; }
.gz-photo__upload-btn {
  display: flex; flex-direction: column; align-items: center; gap: 6px;
  background: rgba(255, 255, 255, 0.12); border: 1px solid rgba(255,255,255,0.28);
  border-radius: 18px; padding: 18px 24px; color: white; font-size: 13px; font-weight: 600;
  backdrop-filter: blur(8px);
}

.gz-photo__float {
  position: absolute; font-family: 'Cormorant Garamond', serif; color: rgba(195, 148, 55, 0.44);
  animation: gz-float 4.5s ease-in-out infinite;
}
.gz-photo__float--a { top: -20px; right: 10px; font-size: 28px; animation-delay: 0s; }
.gz-photo__float--b { bottom: -16px; left: -12px; font-size: 34px; animation-delay: 1s; }
.gz-photo__float--c { top: 42%; right: -22px; font-size: 22px; animation-delay: 1.8s; }
@keyframes gz-float { 0%, 100% { transform: translateY(0) rotate(0deg); } 50% { transform: translateY(-14px) rotate(8deg); } }
.gz-spin { display: inline-block; animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

/* ═══════════════════ SECTION HEADINGS ═══════════════════ */
.gz-section-head { text-align: center; margin-bottom: 64px; }
.gz-section-head__ornaments { display: flex; align-items: center; justify-content: center; gap: 18px; margin-bottom: 20px; }
.gz-section-head__rule { width: 56px; height: 1px; }
.gz-section-head__rule--left { background: linear-gradient(90deg, transparent, rgba(196, 148, 58, 0.55)); }
.gz-section-head__rule--right { background: linear-gradient(270deg, transparent, rgba(196, 148, 58, 0.55)); }
.gz-section-head__floret { font-size: 28px; color: var(--amber-500); font-family: 'Cormorant Garamond', serif; line-height: 1; }
.gz-section-head__title {
  font-family: 'Cormorant Garamond', serif;
  font-size: clamp(34px, 5.5vw, 58px); font-weight: 700; line-height: 1.1;
  letter-spacing: -0.02em; color: var(--stone-900); margin-bottom: 14px;
}
.gz-section-head__sub { font-family: 'DM Sans', sans-serif; font-size: 16px; color: var(--stone-400); line-height: 1.65; max-width: 540px; margin: 0 auto; }

/* ═══════════════════ GENRES ═══════════════════ */
.gz-genres {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(270px, 1fr)); gap: 22px;
}
@media (min-width: 1024px) { .gz-genres { grid-template-columns: repeat(4, 1fr); } }

.gz-genre {
  position: relative; height: 390px; border-radius: 28px; overflow: hidden;
  cursor: pointer;
  transition: transform 0.5s cubic-bezier(0.22, 1, 0.36, 1), box-shadow 0.5s;
  animation: gz-fade-up 0.7s cubic-bezier(0.22, 1, 0.36, 1) both;
  box-shadow: 0 8px 32px rgba(0,0,0,0.12), 0 2px 8px rgba(0,0,0,0.08);
}
@keyframes gz-fade-up { from { opacity: 0; transform: translateY(28px); } to { opacity: 1; transform: translateY(0); } }
.gz-genre:hover { transform: translateY(-14px) scale(1.025); box-shadow: 0 28px 70px rgba(0,0,0,0.22), 0 8px 20px rgba(0,0,0,0.12); }

.gz-genre__bg { position: absolute; inset: 0; transition: transform 0.55s cubic-bezier(0.22, 1, 0.36, 1); }
.gz-genre:hover .gz-genre__bg { transform: scale(1.09); }

.gz-genre__symbols { position: absolute; inset: 0; opacity: 0.14; pointer-events: none; }
.gz-genre__sym {
  position: absolute; font-family: 'Cormorant Garamond', serif; font-size: 26px; color: white;
  animation: gz-sym-float 6.5s ease-in-out infinite;
}
@keyframes gz-sym-float { 0%, 100% { transform: translate(0, 0) rotate(0deg); opacity: 0.25; } 50% { transform: translate(12px, -18px) rotate(18deg); opacity: 0.6; } }

.gz-genre__body { position: relative; z-index: 2; height: 100%; padding: 28px; display: flex; flex-direction: column; }
.gz-genre__top { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 0; }

.gz-genre__icon-wrap {
  position: relative; width: 74px; height: 74px; border-radius: 20px;
  background: rgba(255,255,255,0.94); backdrop-filter: blur(12px);
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 8px 26px rgba(0,0,0,0.16), inset 0 1px 0 white;
  transition: transform 0.5s cubic-bezier(0.22, 1, 0.36, 1), box-shadow 0.5s;
}
.gz-genre:hover .gz-genre__icon-wrap { transform: scale(1.14) rotate(-9deg); box-shadow: 0 14px 40px rgba(0,0,0,0.24); }
.gz-genre__icon-glow { position: absolute; inset: -10px; border-radius: 24px; background: var(--g-glow); filter: blur(14px); opacity: 0; transition: opacity 0.5s; }
.gz-genre:hover .gz-genre__icon-glow { opacity: 1; }
.gz-genre__icon { font-size: 36px; position: relative; z-index: 1; }

.gz-genre__idx {
  font-family: 'Cormorant Garamond', serif; font-size: 52px; font-weight: 700;
  color: rgba(255,255,255,0.22); line-height: 1; transition: all 0.4s;
}
.gz-genre:hover .gz-genre__idx { color: rgba(255,255,255,0.38); transform: scale(1.08); }

.gz-genre__info { flex: 1; display: flex; flex-direction: column; justify-content: flex-end; }
.gz-genre__name {
  font-family: 'Cormorant Garamond', serif; font-size: 34px; font-weight: 700;
  color: white; text-shadow: 0 3px 14px rgba(0,0,0,0.28); margin-bottom: 8px;
  transition: transform 0.4s;
}
.gz-genre:hover .gz-genre__name { transform: translateX(4px); }
.gz-genre__sub {
  font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.3em;
  color: rgba(255,255,255,0.82); text-shadow: 0 1px 4px rgba(0,0,0,0.22); margin-bottom: 12px;
}
.gz-genre__desc {
  font-size: 13px; line-height: 1.65; color: rgba(255,255,255,0.9);
  text-shadow: 0 1px 6px rgba(0,0,0,0.22); opacity: 0; transform: translateY(10px);
  transition: all 0.4s 0.1s;
}
.gz-genre:hover .gz-genre__desc { opacity: 1; transform: translateY(0); }

.gz-genre__stave { position: absolute; bottom: 22px; left: 28px; right: 28px; display: flex; flex-direction: column; gap: 6px; opacity: 0.28; pointer-events: none; transition: opacity 0.4s; }
.gz-genre:hover .gz-genre__stave { opacity: 0.52; }
.gz-genre__stave-line { height: 1.5px; background: white; border-radius: 1px; }

.gz-genre__arrow {
  position: absolute; bottom: 28px; right: 28px;
  font-family: 'Cormorant Garamond', serif; font-size: 30px; color: white;
  opacity: 0; transform: translateX(-10px); transition: all 0.38s 0.15s;
}
.gz-genre:hover .gz-genre__arrow { opacity: 0.82; transform: translateX(0); }
.gz-genre__bloom { position: absolute; inset: 0; background: radial-gradient(circle at center, var(--g-glow) 0%, transparent 68%); opacity: 0; transition: opacity 0.5s; pointer-events: none; }
.gz-genre:hover .gz-genre__bloom { opacity: 0.38; }

/* ═══════════════════ DIVIDER ═══════════════════ */
.gz-divider { display: flex; align-items: center; gap: 16px; margin-bottom: 52px; }
.gz-divider__ornament { font-size: 24px; color: var(--amber-500); font-family: 'Cormorant Garamond', serif; line-height: 1; flex-shrink: 0; }
.gz-divider__line { height: 1px; flex: 1; background: linear-gradient(90deg, rgba(196, 148, 58, 0.35), transparent); }
.gz-divider__label { font-size: 10px; text-transform: uppercase; letter-spacing: 0.44em; color: var(--stone-400); font-weight: 700; flex-shrink: 0; }

/* ═══════════════════ ARCHIVE GRID ═══════════════════ */
.gz-archive-grid { display: grid; grid-template-columns: 5fr 7fr; gap: 26px; align-items: start; }
@media (max-width: 1024px) { .gz-archive-grid { grid-template-columns: 1fr; } }

/* Info panel */
.gz-archive-info { }
.gz-archive-info__inner {
  border-radius: 28px; padding: 38px;
  background: var(--card-bg);
  border: 1.5px solid var(--card-border);
  box-shadow: var(--shadow-md);
  backdrop-filter: blur(18px);
  display: flex; flex-direction: column; gap: 22px;
}
.gz-archive-info__header { display: flex; align-items: center; justify-content: space-between; }
.gz-archive-info__label { display: flex; align-items: center; gap: 10px; }
.gz-archive-info__clef { font-family: 'Cormorant Garamond', serif; font-size: 22px; color: var(--amber-500); line-height: 1; }
.gz-archive-info__title-text { font-size: 10px; text-transform: uppercase; letter-spacing: 0.38em; color: var(--stone-400); font-weight: 700; }
.gz-edit-btn {
  font-size: 11px; text-transform: uppercase; letter-spacing: 0.12em; font-weight: 700;
  color: var(--amber-600); border: none; background: none; cursor: pointer; padding: 8px 14px; border-radius: 10px;
  transition: background 0.2s, color 0.2s;
}
.gz-edit-btn:hover { background: rgba(248, 222, 168, 0.55); color: var(--amber-800); }

.gz-archive-headline {
  font-family: 'Cormorant Garamond', serif;
  font-size: clamp(20px, 2.4vw, 28px); font-weight: 600; line-height: 1.35;
  color: var(--stone-900); letter-spacing: -0.01em;
}
.gz-archive-headline-edit {
  font-family: 'Cormorant Garamond', serif;
  font-size: clamp(20px, 2.4vw, 28px); font-weight: 600; line-height: 1.35;
  color: var(--stone-900); letter-spacing: -0.01em;
  outline: none; border-radius: 8px; padding: 6px 10px; margin: -6px -10px;
  white-space: pre-wrap; word-break: break-word; overflow: hidden;
  transition: background 0.2s, box-shadow 0.2s; cursor: text;
}
.gz-archive-headline-edit:hover { background: rgba(255, 238, 200, 0.38); }
.gz-archive-headline-edit:focus { background: rgba(255, 238, 200, 0.58); box-shadow: 0 0 0 2px rgba(196, 122, 58, 0.28); }

.gz-archive-desc { font-family: 'DM Sans', sans-serif; font-size: 15px; color: var(--stone-500); line-height: 1.78; }
.gz-archive-desc-edit {
  font-family: 'DM Sans', sans-serif; font-size: 15px; color: var(--stone-500); line-height: 1.78;
  outline: none; border-radius: 8px; padding: 6px 10px; margin: -6px -10px;
  white-space: pre-wrap; word-break: break-word; overflow: hidden; cursor: text;
  transition: background 0.2s, box-shadow 0.2s;
}
.gz-archive-desc-edit:hover { background: rgba(255, 238, 200, 0.3); }
.gz-archive-desc-edit:focus { background: rgba(255, 238, 200, 0.46); box-shadow: 0 0 0 2px rgba(196, 122, 58, 0.22); }

.gz-archive-cta-row { display: flex; flex-wrap: wrap; align-items: center; gap: 18px; padding-top: 6px; }
.gz-archive-years { font-size: 10px; text-transform: uppercase; letter-spacing: 0.32em; color: var(--stone-400); font-weight: 700; }

/* Mini stats */
.gz-mini-stats {
  display: flex; align-items: stretch;
  border-top: 1px solid rgba(210, 185, 145, 0.3); padding-top: 22px; gap: 0;
}
.gz-mini-stat { flex: 1; text-align: center; padding: 14px 8px; }
.gz-mini-stat__val { display: block; font-family: 'Cormorant Garamond', serif; font-size: 28px; font-weight: 700; color: var(--stone-800); line-height: 1; }
.gz-mini-stat__lbl { display: block; font-size: 9px; text-transform: uppercase; letter-spacing: 0.28em; color: var(--stone-400); font-weight: 700; margin-top: 5px; }
.gz-mini-stat--gold .gz-mini-stat__val { color: var(--amber-600); }
.gz-mini-stat__sep { width: 1px; align-self: stretch; background: linear-gradient(180deg, transparent, rgba(200, 168, 110, 0.35), transparent); margin: 0 6px; }

/* ═══════════════════ FEATURED ═══════════════════ */
.gz-archive-right { display: flex; flex-direction: column; gap: 20px; }
.gz-featured {
  border-radius: 28px; padding: 32px;
  background: rgba(255, 255, 255, 0.72);
  border: 1.5px solid var(--card-border);
  box-shadow: var(--shadow-sm);
  backdrop-filter: blur(12px);
}
.gz-featured__header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
.gz-featured__title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.36em; color: var(--stone-400); font-weight: 700; }
.gz-featured__all { font-size: 10px; text-transform: uppercase; letter-spacing: 0.28em; color: var(--stone-300); font-weight: 700; cursor: pointer; transition: color 0.22s; }
.gz-featured__all:hover { color: var(--amber-500); }

.gz-featured__grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; }
.gz-feat-note {
  padding: 18px; border-radius: 18px;
  background: rgba(253, 251, 247, 0.85);
  border: 1.5px solid rgba(210, 182, 136, 0.35);
  text-align: left; cursor: pointer;
  transition: all 0.28s cubic-bezier(0.22, 1, 0.36, 1);
  display: flex; flex-direction: column; gap: 10px;
}
.gz-feat-note:hover { background: white; border-color: rgba(196, 122, 58, 0.45); transform: translateY(-4px); box-shadow: 0 14px 36px rgba(110, 66, 10, 0.13); }
.gz-feat-note__type { display: flex; align-items: center; gap: 8px; }
.gz-feat-note__icon {
  width: 28px; height: 28px; border-radius: 8px;
  display: flex; align-items: center; justify-content: center;
  font-family: 'Cormorant Garamond', serif; font-size: 14px; flex-shrink: 0;
}
.gz-feat-note__kind { font-size: 10px; text-transform: uppercase; letter-spacing: 0.2em; font-weight: 700; }
.gz-feat-note__text {
  font-family: 'DM Sans', sans-serif; font-size: 13px; color: var(--stone-500);
  line-height: 1.6; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;
  transition: color 0.28s; flex: 1;
}
.gz-feat-note:hover .gz-feat-note__text { color: var(--stone-800); }
.gz-feat-note__date { font-size: 10px; text-transform: uppercase; letter-spacing: 0.2em; color: var(--stone-300); font-weight: 600; }

.gz-featured__empty {
  display: flex; flex-direction: column; align-items: center; padding: 44px 24px;
  border-radius: 16px; background: rgba(253, 249, 244, 0.62);
  border: 1.5px dashed rgba(215, 188, 148, 0.5); text-align: center;
}
.gz-featured__empty-clef { font-family: 'Cormorant Garamond', serif; font-size: 36px; color: rgba(196, 148, 58, 0.45); line-height: 1; margin-bottom: 10px; }
.gz-featured__empty p { font-size: 13px; color: var(--stone-400); }
.gz-featured__empty-hint { margin-top: 4px; font-size: 11px; color: var(--stone-300) !important; }

/* Select panel */
.gz-select-panel { margin-top: 22px; padding-top: 22px; border-top: 1px solid rgba(210, 182, 136, 0.3); }
.gz-select-panel__title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.36em; color: var(--stone-400); font-weight: 700; margin-bottom: 14px; }
.gz-select-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: 8px;
  max-height: 210px; overflow-y: auto; padding-right: 4px;
}
.gz-select-grid::-webkit-scrollbar { width: 4px; }
.gz-select-grid::-webkit-scrollbar-thumb { background: rgba(196, 122, 58, 0.26); border-radius: 2px; }
.gz-select-item {
  display: flex; align-items: center; gap: 8px; padding: 10px 14px; border-radius: 12px;
  background: rgba(253, 250, 245, 0.82); border: 1.5px solid rgba(210, 182, 136, 0.32);
  cursor: pointer; transition: border-color 0.2s, background 0.2s;
}
.gz-select-item:hover { border-color: rgba(196, 122, 58, 0.44); background: rgba(255, 247, 232, 0.9); }
.gz-select-item__check { accent-color: var(--amber-500); flex-shrink: 0; }
.gz-select-item__text { font-size: 13px; color: var(--stone-500); display: -webkit-box; -webkit-line-clamp: 1; -webkit-box-orient: vertical; overflow: hidden; }

/* ═══════════════════ THOUGHTS ═══════════════════ */
.gz-thoughts {
  border-radius: 24px; padding: 28px 32px;
  background: linear-gradient(135deg, rgba(255, 249, 236, 0.88) 0%, rgba(255, 255, 255, 0.68) 100%);
  border: 1.5px solid rgba(210, 182, 136, 0.34);
  box-shadow: 0 4px 22px rgba(110, 66, 10, 0.05);
  backdrop-filter: blur(10px);
}
.gz-thoughts__header { display: flex; align-items: center; gap: 12px; margin-bottom: 18px; }
.gz-thoughts__ornament { font-size: 20px; color: var(--amber-500); font-family: 'Cormorant Garamond', serif; line-height: 1; }
.gz-thoughts__title { font-size: 10px; text-transform: uppercase; letter-spacing: 0.38em; color: var(--stone-400); font-weight: 700; }
.gz-thoughts__grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; }
.gz-thought {
  padding: 16px 18px; border-radius: 16px; position: relative;
  background: linear-gradient(135deg, rgba(255, 244, 218, 0.75) 0%, rgba(255, 251, 240, 0.52) 100%);
  border: 1px solid rgba(210, 175, 110, 0.28);
}
.gz-thought__quote-mark {
  font-family: 'Cormorant Garamond', serif; font-size: 52px; font-weight: 300;
  color: rgba(196, 148, 58, 0.22); line-height: 1; position: absolute; top: 6px; left: 14px;
}
.gz-thought__text {
  font-family: 'Cormorant Garamond', serif; font-size: 14px; font-style: italic;
  color: var(--stone-500); line-height: 1.72; position: relative; z-index: 1; padding-top: 22px;
  display: -webkit-box; -webkit-line-clamp: 4; -webkit-box-orient: vertical; overflow: hidden;
}

/* ═══════════════════ COLLECTION ═══════════════════ */
.gz-collection-head { display: flex; align-items: center; margin-bottom: 52px; gap: 18px; }
.gz-collection-head__eyebrow { font-size: 10px; text-transform: uppercase; letter-spacing: 0.44em; color: var(--stone-400); font-weight: 700; margin-bottom: 8px; }
.gz-collection-head__accent-bar { height: 3px; width: 52px; border-radius: 100px; background: linear-gradient(90deg, var(--amber-500), var(--amber-300)); }
.gz-collection-head__rule { height: 1px; flex: 1; background: linear-gradient(90deg, rgba(210, 182, 136, 0.55), transparent); }
.gz-collection-head__count { font-family: 'JetBrains Mono', monospace; font-size: 12px; color: var(--stone-400); white-space: nowrap; }

.gz-notes-grid { display: grid; grid-template-columns: repeat(12, 1fr); gap: 20px; }

/* Empty */
.gz-empty { text-align: center; padding: 120px 24px; }
.gz-empty__card {
  display: inline-flex; flex-direction: column; align-items: center;
  padding: 64px 88px; border-radius: 36px;
  background: rgba(255, 255, 255, 0.74); backdrop-filter: blur(22px);
  border: 1.5px solid var(--card-border); box-shadow: var(--shadow-lg);
}
.gz-empty__clef { font-family: 'Cormorant Garamond', serif; font-size: 88px; color: rgba(196, 148, 58, 0.4); line-height: 1; margin-bottom: 24px; display: block; }
.gz-empty__title { font-family: 'Cormorant Garamond', serif; font-size: 34px; font-weight: 600; color: var(--stone-700); margin-bottom: 12px; }
.gz-empty__body { font-size: 15px; color: var(--stone-400); line-height: 1.68; max-width: 340px; margin-bottom: 36px; }

/* Loading */
.gz-loading { text-align: center; padding: 120px 24px; display: flex; flex-direction: column; align-items: center; gap: 18px; }
.gz-loading__ring { position: relative; width: 60px; height: 60px; }
.gz-loading__ping { position: absolute; inset: 0; border-radius: 50%; border: 2px solid rgba(196, 148, 58, 0.25); animation: gz-ping 1.4s ease-out infinite; }
.gz-loading__spin { position: absolute; inset: 0; border-radius: 50%; border: 2px solid transparent; border-top-color: var(--amber-600); border-right-color: rgba(196, 148, 58, 0.3); animation: gz-spin 0.9s linear infinite; }
@keyframes gz-ping { 0% { transform: scale(1); opacity: 0.6; } 100% { transform: scale(1.6); opacity: 0; } }
@keyframes gz-spin { to { transform: rotate(360deg); } }
.gz-loading__label { font-size: 10px; text-transform: uppercase; letter-spacing: 0.42em; color: var(--stone-400); font-weight: 700; }

/* Load more */
.gz-load-more { text-align: center; margin-top: 64px; }

/* ═══════════════════ INLINE EDIT ═══════════════════ */
.gz-inline-edit {
  background: transparent; outline: none; border: none;
  border-bottom: 1.5px solid transparent; font: inherit; color: inherit;
  transition: border-color 0.22s;
}
.gz-inline-edit:hover { border-bottom-color: rgba(196, 148, 58, 0.4); }
.gz-inline-edit:focus { border-bottom-color: var(--amber-500); }
.gz-inline-edit--label { font-size: 10px; text-transform: uppercase; letter-spacing: 0.38em; color: var(--stone-400); font-weight: 700; }
.gz-inline-edit--cta { color: white; width: 128px; font-size: 13px; font-weight: 600; }
.gz-inline-edit--footer-year { width: 40px; text-align: center; }

/* ═══════════════════ FOOTER ═══════════════════ */
.gz-footer {
  border-top: 1px solid rgba(210, 182, 136, 0.28);
  padding: 52px 48px;
  background: linear-gradient(180deg, transparent 0%, rgba(242, 228, 198, 0.22) 100%);
}
.gz-footer__inner {
  max-width: 1280px; margin: 0 auto;
  display: flex; align-items: center; justify-content: space-between; gap: 32px;
  flex-wrap: wrap;
}
.gz-footer__brand { display: flex; align-items: center; gap: 18px; }
.gz-footer__clef { font-family: 'Cormorant Garamond', serif; font-size: 46px; color: var(--amber-500); line-height: 1; }
.gz-footer__name { font-family: 'Cormorant Garamond', serif; font-size: 26px; font-weight: 700; color: var(--stone-800); line-height: 1; }
.gz-footer__years { font-size: 9px; text-transform: uppercase; letter-spacing: 0.4em; color: var(--stone-400); font-weight: 700; display: flex; align-items: center; gap: 4px; margin-top: 4px; }
.gz-footer__quote {
  font-family: 'Cormorant Garamond', serif; font-size: 18px; font-style: italic;
  color: var(--stone-400); line-height: 1.65; text-align: right; max-width: 440px;
}
.gz-footer__quote--edit {
  font-family: 'Cormorant Garamond', serif; font-size: 18px; font-style: italic;
  color: var(--stone-400); line-height: 1.65; text-align: right; max-width: 440px;
  outline: none; border-radius: 8px; padding: 4px 8px; margin: -4px -8px;
  transition: background 0.2s;
}
.gz-footer__quote--edit:hover { background: rgba(255, 238, 200, 0.3); }
.gz-footer__quote--edit:focus { background: rgba(255, 238, 200, 0.5); }

/* ═══════════════════ SCROLLBAR ═══════════════════ */
::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: rgba(242, 228, 200, 0.45); }
::-webkit-scrollbar-thumb { background: rgba(196, 122, 58, 0.24); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: rgba(196, 122, 58, 0.45); }
</style>