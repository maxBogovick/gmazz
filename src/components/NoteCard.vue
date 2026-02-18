<script setup lang="ts">
import { computed } from 'vue';
import type { Note, NoteType } from '../types';
import ThoughtCard from './notes/ThoughtCard.vue';
import PhraseCard from './notes/PhraseCard.vue';
import RhythmCard from './notes/RhythmCard.vue';
import ScoreCard from './notes/ScoreCard.vue';

const props = defineProps<{
  note: Note;
  index: number;
  currentPlayingId: string | null;
}>();

const emit = defineEmits<{
  (e: 'open', note: Note): void;
  (e: 'togglePlayback', event: Event, note: Note): void;
}>();

/* ─── Type Config ─────────────────────────────────────── */
const typeConfig: Record<NoteType, {
  name: string; icon: string; symbol: string;
  accent: string; accentDark: string;
  bg: string; bgStrong: string;
  tag: string; tagText: string;
}> = {
  thought: {
    name: 'Мысль', icon: '✦', symbol: '✦',
    accent: '#D4760A', accentDark: '#8B4A00',
    bg: '#FFFBF2', bgStrong: '#FEF3D6',
    tag: '#FDE89A', tagText: '#7A4400',
  },
  phrase: {
    name: 'Фраза', icon: '𝄞', symbol: '♪',
    accent: '#C0441E', accentDark: '#7A2206',
    bg: '#FFF8F5', bgStrong: '#FFE9DE',
    tag: '#FFCCB0', tagText: '#7A2206',
  },
  harmony: {
    name: 'Гармония', icon: '♬', symbol: '♬',
    accent: '#0E7E6A', accentDark: '#074D40',
    bg: '#F2FDFB', bgStrong: '#D6F5EF',
    tag: '#A5EBE0', tagText: '#074D40',
  },
  rhythm: {
    name: 'Ритм', icon: '♩', symbol: '♩',
    accent: '#9B3EC0', accentDark: '#5E1880',
    bg: '#FDF5FF', bgStrong: '#F2DAFF',
    tag: '#E2B8F7', tagText: '#5E1880',
  },
  score: {
    name: 'Партитура', icon: '𝄚', symbol: '≡',
    accent: '#1E62C0', accentDark: '#0C3878',
    bg: '#F3F8FF', bgStrong: '#D8E9FF',
    tag: '#B0CFFF', tagText: '#0C3878',
  },
};

/* ─── Card sizing ─────────────────────────────────────── */
const cardSize = computed(() => {
  const p = props.index % 7;
  return {
    cols: p === 0
        ? 'col-span-12 md:col-span-8'
        : (p === 5 || p === 6)
            ? 'col-span-12 md:col-span-6'
            : 'col-span-12 md:col-span-4',
    isLarge: p === 0,
    minHeight: p === 0 ? 'min-h-[320px]' : 'min-h-[230px]',
  };
});

/* ─── Playing state ───────────────────────────────────── */
const isPlaying = computed(() => props.currentPlayingId === props.note.id);

const hasAudio = computed(() =>
    (props.note.note_type === 'phrase' && props.note.metadata?.file_path) ||
    (props.note.note_type === 'score' && props.note.metadata?.audio_path)
);

// 24 bars for waveform visualization — varied heights for organic look
const waveformBars = [38, 62, 48, 75, 55, 88, 44, 70, 58, 92, 40, 66,
  52, 80, 46, 72, 60, 85, 42, 68, 56, 78, 50, 64];

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric', month: 'short', year: 'numeric',
  });
}

function getCardChips(note: Note): string[] {
  const chips: string[] = [];
  if (note.note_type === 'thought') {
    if (note.metadata?.tags?.length)
      chips.push(...note.metadata.tags.slice(0, 2).map((t: string) => `#${t}`));
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
  <article
      @click="emit('open', note)"
      class="group cursor-pointer"
      :class="cardSize.cols"
  >
    <div
        class="note-card h-full flex flex-col relative overflow-hidden"
        :class="[cardSize.minHeight, { 'is-playing': isPlaying }]"
        :style="{
          '--accent':      typeConfig[note.note_type].accent,
          '--accent-dark': typeConfig[note.note_type].accentDark,
          '--bg':          typeConfig[note.note_type].bg,
          '--bg-strong':   typeConfig[note.note_type].bgStrong,
          '--tag':         typeConfig[note.note_type].tag,
          '--tag-text':    typeConfig[note.note_type].tagText,
        }"
    >
      <!-- ── Left colour stripe ──────────────────────── -->
      <div class="card-stripe" />

      <!-- ── Background decorative glyph ───────────── -->
      <div class="card-bg-symbol" aria-hidden="true">
        {{ typeConfig[note.note_type].symbol }}
      </div>

      <!-- ── Hover shimmer ──────────────────────────── -->
      <div class="card-shimmer" />

      <!-- ── Playing aurora glow (phrase + score) ───── -->
      <div v-if="hasAudio" class="card-aurora" />

      <!-- ── Sonar rings (active on play) ───────────── -->
      <div v-if="hasAudio && isPlaying" class="sonar-rings" aria-hidden="true">
        <span class="sonar-ring ring-1" />
        <span class="sonar-ring ring-2" />
        <span class="sonar-ring ring-3" />
      </div>

      <!-- ─────────────── CONTENT ─────────────────── -->
      <div class="relative z-10 h-full flex flex-col px-6 py-6 lg:px-7 lg:py-7">

        <!-- Header -->
        <div class="flex items-start gap-3 mb-5">

          <!-- Animated icon badge -->
          <div class="card-icon-badge shrink-0">
            <!-- Spinning vinyl ring when playing -->
            <span v-if="isPlaying && note.note_type === 'phrase'" class="vinyl-ring" aria-hidden="true" />
            <span class="card-icon-glyph">{{ typeConfig[note.note_type].icon }}</span>
          </div>

          <!-- Type name + date -->
          <div class="flex-1 min-w-0 mt-0.5">
            <div class="card-type-label">{{ typeConfig[note.note_type].name }}</div>
            <div class="card-date">{{ formatDate(note.created_at) }}</div>
          </div>

          <!-- Arrow hint (appears on hover) -->
          <div class="card-arrow" aria-hidden="true">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2.5" stroke-linecap="round"
                 stroke-linejoin="round">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </div>
        </div>

        <!-- Chips -->
        <div v-if="getCardChips(note).length" class="flex flex-wrap gap-1.5 mb-4">
          <span
              v-for="(chip, ci) in getCardChips(note)"
              :key="ci"
              class="card-chip"
          >{{ chip }}</span>
        </div>

        <!-- Note content -->
        <div class="flex-1 min-h-0 overflow-hidden relative">
          <ThoughtCard v-if="note.note_type === 'thought'" :note="note" />

          <!-- ═══════════════════════════════════════════
               PHRASE — waveform + play circle overlay
          ═══════════════════════════════════════════════ -->
          <div
              v-else-if="note.note_type === 'phrase'"
              class="phrase-player h-full flex flex-col"
              :class="{ 'phrase-player--playing': isPlaying }"
          >
            <!-- Waveform visualizer -->
            <div class="waveform-stage" :class="{ 'waveform-stage--active': isPlaying }">
              <div class="waveform-track">
                <span
                    v-for="(h, bi) in waveformBars"
                    :key="bi"
                    class="waveform-bar"
                    :style="{
                    '--bar-h': `${h}%`,
                    '--bar-delay': `${(bi * 47) % 600}ms`,
                  }"
                />
              </div>
              <!-- Scanning line that sweeps across when playing -->
              <div v-if="isPlaying" class="waveform-scan-line" />
              <!-- Glow beneath the bars -->
              <div class="waveform-floor-glow" />

              <!-- Circle play button inside waveform -->
              <button
                  v-if="note.metadata?.file_path"
                  @click.stop="emit('togglePlayback', $event, note)"
                  class="phrase-circle-btn"
                  :class="{ 'phrase-circle-btn--playing': isPlaying }"
                  :title="isPlaying ? 'Пауза' : 'Прослушать'"
              >
                <span class="phrase-circle-pulse" />
                <span class="phrase-circle-icon">
                  <svg v-if="!isPlaying" viewBox="0 0 24 24" fill="currentColor" width="14" height="14">
                    <path d="M8 5.14v14l11-7-11-7z"/>
                  </svg>
                  <span v-else class="pause-bars-icon"><span /><span /></span>
                </span>
              </button>
            </div>

            <!-- Original PhraseCard content -->
            <div class="phrase-meta-layer flex-1">
              <PhraseCard :note="note" />
            </div>
          </div>

          <RhythmCard  v-else-if="note.note_type === 'rhythm'"  :note="note" />

          <!-- Score with Play Button Overlay -->
          <div v-else-if="note.note_type === 'score'" class="h-full relative group/score">
            <ScoreCard :note="note" />
            <button
                v-if="note.metadata?.audio_path"
                @click.stop="emit('togglePlayback', $event, note)"
                class="score-play-btn"
                :class="{ 'score-play-btn--playing': isPlaying }"
                title="Прослушать запись"
            >
              <span class="score-play-ring" />
              <span class="text-lg leading-none" :style="{ color: typeConfig[note.note_type].accent }">
                {{ isPlaying ? '⏸' : '▶' }}
              </span>
            </button>
          </div>
        </div>

        <!-- Note content text -->
        <div v-if="note.content && note.note_type !== 'thought'" class="note-content-preview mt-4">
          <p class="note-content-text">{{ note.content }}</p>
        </div>

        <!-- Footer -->
        <div class="card-footer mt-5 pt-4">
          <span class="card-id">#{{ note.id.substring(0, 8) }}</span>
          <button class="card-open-btn" @click.stop="emit('open', note)">
            <span>открыть</span>
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2.5" stroke-linecap="round"
                 stroke-linejoin="round">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </article>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,600;0,700;1,600&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap');

/* ───────────────────────────────────────────────────────
   CARD SHELL
─────────────────────────────────────────────────────── */
.note-card {
  font-family: 'Plus Jakarta Sans', sans-serif;
  border-radius: 20px;
  background: var(--bg);
  border: 1.5px solid color-mix(in srgb, var(--accent) 18%, #ede7de);
  box-shadow:
      0 2px 8px color-mix(in srgb, var(--accent) 8%, transparent),
      0 1px 3px rgb(0 0 0 / 0.05);
  transition:
      transform 0.36s cubic-bezier(0.34, 1.56, 0.64, 1),
      box-shadow 0.32s ease,
      border-color 0.28s ease;
}

.note-card:hover {
  transform: translateY(-6px) scale(1.016);
  border-color: color-mix(in srgb, var(--accent) 38%, transparent);
  box-shadow:
      0 2px 4px   color-mix(in srgb, var(--accent) 10%, rgb(0 0 0 / 0.04)),
      0 8px 16px  color-mix(in srgb, var(--accent) 12%, rgb(0 0 0 / 0.05)),
      0 20px 40px color-mix(in srgb, var(--accent) 16%, rgb(0 0 0 / 0.07)),
      0 40px 70px -10px color-mix(in srgb, var(--accent) 22%, rgb(0 0 0 / 0.08));
}

/* ── Playing state: card glows and breathes ──────────── */
.note-card.is-playing {
  border-color: color-mix(in srgb, var(--accent) 55%, transparent);
  box-shadow:
      0 0 0 3px color-mix(in srgb, var(--accent) 12%, transparent),
      0 0 32px color-mix(in srgb, var(--accent) 22%, transparent),
      0 4px 20px color-mix(in srgb, var(--accent) 15%, transparent);
  animation: card-breathe 2.4s ease-in-out infinite;
}

@keyframes card-breathe {
  0%, 100% {
    box-shadow:
        0 0 0 3px color-mix(in srgb, var(--accent) 12%, transparent),
        0 0 32px color-mix(in srgb, var(--accent) 22%, transparent),
        0 4px 20px color-mix(in srgb, var(--accent) 15%, transparent);
  }
  50% {
    box-shadow:
        0 0 0 5px color-mix(in srgb, var(--accent) 18%, transparent),
        0 0 52px color-mix(in srgb, var(--accent) 32%, transparent),
        0 8px 36px color-mix(in srgb, var(--accent) 22%, transparent);
  }
}

/* ───────────────────────────────────────────────────────
   LEFT ACCENT STRIPE
─────────────────────────────────────────────────────── */
.card-stripe {
  position: absolute;
  inset-block: 0;
  left: 0;
  width: 4px;
  border-radius: 20px 0 0 20px;
  background: linear-gradient(
      180deg,
      var(--accent) 0%,
      color-mix(in srgb, var(--accent) 45%, transparent) 100%
  );
  transition: width 0.28s ease;
}

.note-card:hover .card-stripe,
.note-card.is-playing .card-stripe { width: 5px; }

/* ── Stripe pulses when playing ──────────────────────── */
.note-card.is-playing .card-stripe {
  animation: stripe-pulse 1.8s ease-in-out infinite;
}

@keyframes stripe-pulse {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.5; }
}

/* ───────────────────────────────────────────────────────
   BACKGROUND MUSIC GLYPH
─────────────────────────────────────────────────────── */
.card-bg-symbol {
  position: absolute;
  bottom: -14px;
  right: 2px;
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 128px;
  line-height: 1;
  color: var(--bg-strong);
  user-select: none;
  pointer-events: none;
  z-index: 0;
  transition:
      transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1),
      color 0.4s ease;
}

.note-card:hover .card-bg-symbol {
  transform: scale(1.14) translateY(-10px);
  color: color-mix(in srgb, var(--bg-strong) 80%, var(--tag));
}

.note-card.is-playing .card-bg-symbol {
  animation: symbol-float 4s ease-in-out infinite;
  color: color-mix(in srgb, var(--bg-strong) 60%, var(--tag));
}

@keyframes symbol-float {
  0%, 100% { transform: scale(1.1) translateY(-4px); }
  50%       { transform: scale(1.14) translateY(-14px); }
}

/* ───────────────────────────────────────────────────────
   SHIMMER GLOW
─────────────────────────────────────────────────────── */
.card-shimmer {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: radial-gradient(
      ellipse 85% 55% at 50% -5%,
      color-mix(in srgb, var(--accent) 14%, transparent) 0%,
      transparent 68%
  );
  opacity: 0;
  pointer-events: none;
  z-index: 1;
  transition: opacity 0.42s ease;
}

.note-card:hover .card-shimmer { opacity: 1; }
.note-card.is-playing .card-shimmer { opacity: 1; }

/* ───────────────────────────────────────────────────────
   AURORA GLOW (playing state, audio cards)
─────────────────────────────────────────────────────── */
.card-aurora {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: radial-gradient(
      ellipse 120% 70% at 50% 110%,
      color-mix(in srgb, var(--accent) 16%, transparent) 0%,
      transparent 60%
  );
  opacity: 0;
  pointer-events: none;
  z-index: 1;
  transition: opacity 0.6s ease;
}

.note-card.is-playing .card-aurora {
  opacity: 1;
  animation: aurora-breathe 3s ease-in-out infinite;
}

@keyframes aurora-breathe {
  0%, 100% { opacity: 0.6; }
  50%       { opacity: 1; }
}

/* ───────────────────────────────────────────────────────
   SONAR RINGS  (playing state)
─────────────────────────────────────────────────────── */
.sonar-rings {
  position: absolute;
  top: 28px;
  left: 28px;
  width: 44px;
  height: 44px;
  z-index: 2;
  pointer-events: none;
}

.sonar-ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  border: 1.5px solid color-mix(in srgb, var(--accent) 50%, transparent);
  animation: sonar-expand 2.4s ease-out infinite;
}

.ring-1 { animation-delay: 0s; }
.ring-2 { animation-delay: 0.7s; }
.ring-3 { animation-delay: 1.4s; }

@keyframes sonar-expand {
  0%   { transform: scale(1); opacity: 0.8; }
  100% { transform: scale(3.5); opacity: 0; }
}

/* ───────────────────────────────────────────────────────
   ICON BADGE
─────────────────────────────────────────────────────── */
.card-icon-badge {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(145deg, var(--bg-strong) 0%, var(--tag) 100%);
  border: 1.5px solid color-mix(in srgb, var(--accent) 22%, transparent);
  box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 22%, transparent);
  position: relative;
  overflow: visible;
  transition:
      transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1),
      box-shadow 0.28s ease;
}

.note-card:hover .card-icon-badge {
  transform: scale(1.12) translateY(-2px);
  box-shadow: 0 8px 22px color-mix(in srgb, var(--accent) 35%, transparent);
}

/* Vinyl spinning ring around icon when phrase is playing */
.vinyl-ring {
  position: absolute;
  inset: -6px;
  border-radius: 50%;
  border: 2px dashed color-mix(in srgb, var(--accent) 60%, transparent);
  animation: vinyl-spin 3s linear infinite;
  pointer-events: none;
}

@keyframes vinyl-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.card-icon-glyph {
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 22px;
  color: var(--accent-dark);
  line-height: 1;
  display: block;
  position: relative;
  z-index: 1;
}

/* ───────────────────────────────────────────────────────
   TYPE LABEL & DATE
─────────────────────────────────────────────────────── */
.card-type-label {
  font-size: 10.5px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: var(--accent);
  line-height: 1.2;
  margin-bottom: 3px;
}

.card-date {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  font-weight: 500;
  color: color-mix(in srgb, var(--accent-dark) 50%, #b0a090);
  letter-spacing: 0.03em;
}

/* ───────────────────────────────────────────────────────
   ARROW HINT
─────────────────────────────────────────────────────── */
.card-arrow {
  color: var(--accent);
  opacity: 0;
  transform: translate(-6px, 4px);
  transition:
      opacity 0.24s ease,
      transform 0.32s cubic-bezier(0.34, 1.56, 0.64, 1);
  flex-shrink: 0;
  margin-top: 2px;
}

.note-card:hover .card-arrow {
  opacity: 0.8;
  transform: translate(0, 0);
}

/* ───────────────────────────────────────────────────────
   CHIPS
─────────────────────────────────────────────────────── */
.card-chip {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 100px;
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  background: var(--tag);
  color: var(--tag-text);
  border: 1px solid color-mix(in srgb, var(--accent) 18%, transparent);
  transition: box-shadow 0.2s ease;
}

.note-card:hover .card-chip {
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent) 20%, transparent);
}

/* ───────────────────────────────────────────────────────
   ██████  PHRASE PLAYER  ██████
─────────────────────────────────────────────────────── */
.phrase-player {
  gap: 10px;
}

/* ── Waveform stage ──────────────────────────────────── */
.waveform-stage {
  position: relative;
  height: 72px;
  border-radius: 12px;
  overflow: hidden;
  background: color-mix(in srgb, var(--bg-strong) 70%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 14%, transparent);
  padding: 8px 10px 6px;
  transition: background 0.4s ease, border-color 0.4s ease;
}

.waveform-stage--active {
  background: color-mix(in srgb, var(--tag) 45%, var(--bg-strong));
  border-color: color-mix(in srgb, var(--accent) 32%, transparent);
}

.waveform-track {
  display: flex;
  align-items: flex-end;
  gap: 2.5px;
  height: 100%;
  width: 100%;
}

/* Each bar */
.waveform-bar {
  flex: 1;
  min-width: 0;
  height: var(--bar-h);
  border-radius: 2px 2px 1px 1px;
  background: color-mix(in srgb, var(--accent) 35%, transparent);
  transition: background 0.4s ease, height 0.2s ease;
  transform-origin: bottom center;
}

/* When playing: bars dance */
.waveform-stage--active .waveform-bar {
  background: linear-gradient(
      to top,
      var(--accent),
      color-mix(in srgb, var(--accent) 55%, #fff)
  );
  animation: bar-dance 0.8s ease-in-out infinite alternate;
  animation-delay: var(--bar-delay);
}

@keyframes bar-dance {
  0%   { transform: scaleY(0.25); opacity: 0.55; }
  30%  { transform: scaleY(1.1);  opacity: 1; }
  65%  { transform: scaleY(0.55); opacity: 0.75; }
  100% { transform: scaleY(0.9);  opacity: 0.92; }
}

/* Scanning progress line */
.waveform-scan-line {
  position: absolute;
  inset-block: 0;
  left: -3px;
  width: 2px;
  background: linear-gradient(
      to bottom,
      transparent 0%,
      var(--accent) 30%,
      var(--accent) 70%,
      transparent 100%
  );
  box-shadow: 0 0 8px 2px color-mix(in srgb, var(--accent) 60%, transparent);
  border-radius: 2px;
  animation: scan-sweep 3s linear infinite;
  pointer-events: none;
  z-index: 5;
}

@keyframes scan-sweep {
  0%   { left: 0%; opacity: 0; }
  5%   { opacity: 1; }
  95%  { opacity: 1; }
  100% { left: 100%; opacity: 0; }
}

/* Floor glow beneath bars */
.waveform-floor-glow {
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 18px;
  background: linear-gradient(
      to top,
      color-mix(in srgb, var(--accent) 14%, transparent),
      transparent
  );
  pointer-events: none;
}

/* ── Phrase meta layer (wraps PhraseCard content) ─────── */
.phrase-meta-layer {
  flex: 1;
  min-h-0: 0;
  overflow: hidden;
}

/* ── Circle play button inside waveform ──────────────── */
.phrase-circle-btn {
  position: absolute;
  top: 50%;
  right: 12px;
  transform: translateY(-50%);
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: rgba(255,255,255,0.92);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1.5px solid color-mix(in srgb, var(--accent) 28%, transparent);
  box-shadow:
      0 3px 10px rgb(0 0 0 / 0.10),
      0 1px 3px rgb(0 0 0 / 0.06);
  cursor: pointer;
  z-index: 10;
  color: var(--accent);
  overflow: visible;
  transition:
      transform 0.32s cubic-bezier(0.34, 1.56, 0.64, 1),
      background 0.24s ease,
      box-shadow 0.24s ease;
}

.phrase-circle-btn:hover {
  transform: translateY(-50%) scale(1.15);
  background: var(--accent);
  color: #fff;
  box-shadow:
      0 6px 20px color-mix(in srgb, var(--accent) 40%, transparent),
      0 2px 6px rgb(0 0 0 / 0.08);
}

.phrase-circle-btn--playing {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
  box-shadow:
      0 0 0 4px color-mix(in srgb, var(--accent) 18%, transparent),
      0 6px 22px color-mix(in srgb, var(--accent) 38%, transparent);
  animation: score-btn-glow 1.8s ease-in-out infinite;
}

.phrase-circle-btn--playing:hover {
  transform: translateY(-50%) scale(1.12);
}

/* Pulsing ring */
.phrase-circle-pulse {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  pointer-events: none;
}

.phrase-circle-btn--playing .phrase-circle-pulse {
  animation: play-pulse-ring 1.6s ease-out infinite;
}

/* Icon wrapper */
.phrase-circle-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
}

/* Animated pause bars icon */
.pause-bars-icon {
  display: flex;
  align-items: center;
  gap: 3px;
}

.pause-bars-icon span {
  display: block;
  width: 3px;
  height: 11px;
  border-radius: 2px;
  background: currentColor;
  animation: pause-bar-bounce 0.6s ease-in-out infinite alternate;
}

.pause-bars-icon span:last-child {
  animation-delay: 0.18s;
}

@keyframes pause-bar-bounce {
  0%   { transform: scaleY(0.5); }
  100% { transform: scaleY(1); }
}

@keyframes play-pulse-ring {
  0%   { box-shadow: 0 0 0 0    color-mix(in srgb, var(--accent) 45%, transparent); }
  70%  { box-shadow: 0 0 0 10px color-mix(in srgb, var(--accent) 0%,  transparent); }
  100% { box-shadow: 0 0 0 0    color-mix(in srgb, var(--accent) 0%,  transparent); }
}

/* ───────────────────────────────────────────────────────
   SCORE PLAY BUTTON (enhanced)
─────────────────────────────────────────────────────── */
.score-play-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: rgba(255,255,255,0.92);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1.5px solid rgb(0 0 0 / 0.08);
  box-shadow:
      0 3px 10px rgb(0 0 0 / 0.10),
      0 1px 3px rgb(0 0 0 / 0.06);
  cursor: pointer;
  z-index: 20;
  overflow: visible;
  transition:
      transform 0.32s cubic-bezier(0.34, 1.56, 0.64, 1),
      box-shadow 0.24s ease;
}

.score-play-btn:hover {
  transform: scale(1.14);
  box-shadow:
      0 6px 20px rgb(0 0 0 / 0.15),
      0 2px 6px rgb(0 0 0 / 0.08);
}
.score-play-btn--playing {
  background: var(--accent);
  border-color: var(--accent);
  box-shadow:
      0 0 0 5px color-mix(in srgb, var(--accent) 20%, transparent),
      0 6px 22px color-mix(in srgb, var(--accent) 35%, transparent);
  animation: score-btn-glow 1.8s ease-in-out infinite;
}

.score-play-btn--playing span.text-lg {
  color: #fff !important;
}

@keyframes score-btn-glow {
  0%, 100% {
    box-shadow:
        0 0 0 5px color-mix(in srgb, var(--accent) 20%, transparent),
        0 6px 22px color-mix(in srgb, var(--accent) 35%, transparent);
  }
  50% {
    box-shadow:
        0 0 0 8px color-mix(in srgb, var(--accent) 14%, transparent),
        0 10px 36px color-mix(in srgb, var(--accent) 48%, transparent);
  }
}

.score-play-ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  pointer-events: none;
}

.score-play-btn--playing .score-play-ring {
  animation: play-pulse-ring 1.6s ease-out infinite;
}

/* ───────────────────────────────────────────────────────
   NOTE CONTENT PREVIEW
─────────────────────────────────────────────────────── */
.note-content-preview {
  position: relative;
  padding: 10px 14px 10px 16px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-strong) 55%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 10%, transparent);
  overflow: hidden;
  transition: background 0.28s ease, border-color 0.28s ease;
}

/* Left quote accent line */
.note-content-preview::before {
  content: '';
  position: absolute;
  left: 0;
  inset-block: 0;
  width: 3px;
  border-radius: 3px 0 0 3px;
  background: linear-gradient(
      180deg,
      var(--accent) 0%,
      color-mix(in srgb, var(--accent) 30%, transparent) 100%
  );
  opacity: 0.5;
  transition: opacity 0.28s ease;
}

/* Decorative open-quote glyph */
.note-content-preview::after {
  content: '\201C';
  position: absolute;
  top: -4px;
  right: 10px;
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 52px;
  line-height: 1;
  color: var(--tag);
  pointer-events: none;
  user-select: none;
  transition: color 0.3s ease;
}

.note-card:hover .note-content-preview {
  background: color-mix(in srgb, var(--bg-strong) 75%, transparent);
  border-color: color-mix(in srgb, var(--accent) 18%, transparent);
}

.note-card:hover .note-content-preview::before {
  opacity: 0.85;
}

.note-card.is-playing .note-content-preview::before {
  opacity: 1;
}

.note-content-text {
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 13px;
  font-weight: 600;
  font-style: italic;
  line-height: 1.65;
  color: color-mix(in srgb, var(--accent-dark) 80%, #3a2e22);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  position: relative;
  z-index: 1;
  letter-spacing: 0.01em;
}

/* ───────────────────────────────────────────────────────
   FOOTER
─────────────────────────────────────────────────────── */
.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid color-mix(in srgb, var(--accent) 13%, #e8e0d4);
}

.card-id {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  font-weight: 500;
  letter-spacing: 0.06em;
  color: color-mix(in srgb, var(--accent-dark) 38%, #b8b0a4);
}

/* ───────────────────────────────────────────────────────
   OPEN BUTTON
─────────────────────────────────────────────────────── */
.card-open-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 13px;
  border-radius: 100px;
  border: 1.5px solid color-mix(in srgb, var(--accent) 32%, transparent);
  background: transparent;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 9.5px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--accent);
  cursor: pointer;
  transition:
      background 0.22s ease,
      border-color 0.22s ease,
      color 0.22s ease,
      transform 0.32s cubic-bezier(0.34, 1.56, 0.64, 1),
      box-shadow 0.22s ease;
}

.note-card:hover .card-open-btn {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
  transform: translateX(3px);
  box-shadow: 0 4px 14px color-mix(in srgb, var(--accent) 45%, transparent);
}
</style>