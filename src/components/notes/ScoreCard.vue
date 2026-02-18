<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import type { Note } from '../../types';
import { getAssetPath } from '../../api/notes';

const props = defineProps<{
  note: Note;
}>();

const imagePath = ref<string>('');
const isPdf = ref(false);
let lastObjectUrl: string | null = null;

async function loadImage() {
  const filePath = props.note.metadata.file_path;

  isPdf.value = false;
  imagePath.value = '';
  if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
    URL.revokeObjectURL(lastObjectUrl);
    lastObjectUrl = null;
  }

  if (!filePath) return;

  if (filePath.toLowerCase().endsWith('.pdf')) {
    isPdf.value = true;
    return;
  }

  try {
    const path = await getAssetPath(filePath);
    imagePath.value = path;
    lastObjectUrl = path;
  } catch (e) {
    console.error('Failed to load score image', e);
  }
}

onMounted(loadImage);
watch(() => props.note.metadata.file_path, loadImage);

onUnmounted(() => {
  if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
    URL.revokeObjectURL(lastObjectUrl);
  }
});
</script>

<template>
  <div class="score-card-root h-full flex flex-col gap-3">

    <!-- ── Preview container ──────────────────────── -->
    <div class="score-preview-wrap flex-1">

      <!-- ─── PDF state ──────────────────────────── -->
      <div v-if="isPdf" class="score-preview score-preview--pdf">
        <!-- Staff lines decoration -->
        <div class="staff-lines" aria-hidden="true">
          <span v-for="n in 5" :key="n" class="staff-line" />
        </div>

        <div class="pdf-inner">
          <div class="pdf-icon-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                 stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="9" y1="13" x2="15" y2="13"/>
              <line x1="9" y1="17" x2="13" y2="17"/>
            </svg>
          </div>
          <span class="pdf-label">PDF Партитура</span>
          <span class="pdf-filename">{{ note.metadata.file_path }}</span>
        </div>
      </div>

      <!-- ─── Image state ────────────────────────── -->
      <div v-else-if="imagePath" class="score-preview score-preview--image">
        <img
            :src="imagePath"
            alt="Score Preview"
            class="score-img"
        />
        <!-- Bottom fog for smooth fade into card background -->
        <div class="score-img-fog" />
      </div>

      <!-- ─── Empty placeholder ─────────────────── -->
      <div v-else class="score-preview score-preview--empty">
        <div class="staff-lines" aria-hidden="true">
          <span v-for="n in 5" :key="n" class="staff-line" />
        </div>
        <span class="empty-glyph" aria-hidden="true">𝄚</span>
      </div>
    </div>

    <!-- ── Key info pill ─────────────────────────── -->
    <div v-if="note.metadata?.key" class="score-key-pill">
      <span class="score-key-dot" />
      <span class="score-key-label">Тональность</span>
      <strong class="score-key-value">{{ note.metadata.key }}</strong>
    </div>

  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,600;0,700;1,600&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap');

/* Inherit accent variables from parent NoteCard */

/* ───────────────────────────────────────────────────────
   PREVIEW WRAPPER
─────────────────────────────────────────────────────── */
.score-preview-wrap {
  min-height: 120px;
  position: relative;
}

.score-preview {
  position: absolute;
  inset: 0;
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--accent, #1E62C0) 15%, transparent);
  background: color-mix(in srgb, var(--bg-strong, #D8E9FF) 60%, transparent);
  transition: border-color 0.28s ease;
}

/* Parent group-hover bubbles down */
:global(.group:hover) .score-preview {
  border-color: color-mix(in srgb, var(--accent, #1E62C0) 30%, transparent);
}

/* ── Staff lines (decorative) ────────────────────────── */
.staff-lines {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 14px 0;
  gap: 0;
  pointer-events: none;
  z-index: 0;
}

.staff-line {
  display: block;
  height: 1px;
  margin: 6px 12px;
  background: color-mix(in srgb, var(--accent, #1E62C0) 12%, transparent);
  border-radius: 1px;
}

/* ── PDF state ───────────────────────────────────────── */
.score-preview--pdf {
  display: flex;
  align-items: center;
  justify-content: center;
}

.pdf-inner {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  text-align: center;
  padding: 0 16px;
}

.pdf-icon-wrap {
  width: 44px;
  height: 44px;
  border-radius: 13px;
  background: linear-gradient(145deg,
  color-mix(in srgb, var(--bg-strong, #D8E9FF) 90%, transparent),
  color-mix(in srgb, var(--tag, #B0CFFF) 70%, transparent)
  );
  border: 1.5px solid color-mix(in srgb, var(--accent, #1E62C0) 22%, transparent);
  box-shadow: 0 4px 14px color-mix(in srgb, var(--accent, #1E62C0) 18%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent, #1E62C0);
  transition: transform 0.36s cubic-bezier(0.34, 1.56, 0.64, 1);
}

:global(.group:hover) .pdf-icon-wrap {
  transform: scale(1.1) rotate(6deg);
}

.pdf-icon-wrap svg {
  width: 22px;
  height: 22px;
}

.pdf-label {
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 9.5px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: var(--accent-dark, #0C3878);
}

.pdf-filename {
  font-family: 'JetBrains Mono', monospace;
  font-size: 8.5px;
  font-weight: 400;
  color: color-mix(in srgb, var(--accent, #1E62C0) 55%, transparent);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 160px;
}

/* ── Image state ─────────────────────────────────────── */
.score-preview--image {
  background: transparent;
}

.score-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: top;
  opacity: 0.88;
  transition: opacity 0.4s ease, transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
  display: block;
}

:global(.group:hover) .score-img {
  opacity: 1;
  transform: scale(1.03);
}

/* Gradient fog at the bottom */
.score-img-fog {
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 40%;
  background: linear-gradient(
      to top,
      color-mix(in srgb, var(--bg, #F3F8FF) 75%, transparent),
      transparent
  );
  pointer-events: none;
}

/* ── Empty placeholder ───────────────────────────────── */
.score-preview--empty {
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-glyph {
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 56px;
  line-height: 1;
  color: color-mix(in srgb, var(--accent, #1E62C0) 16%, transparent);
  position: relative;
  z-index: 1;
  transition: color 0.3s ease, transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
  user-select: none;
}

:global(.group:hover) .empty-glyph {
  color: color-mix(in srgb, var(--accent, #1E62C0) 28%, transparent);
  //transform: scale(1.1) rotate(-5deg);
}

/* ───────────────────────────────────────────────────────
   KEY PILL
─────────────────────────────────────────────────────── */
.score-key-pill {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 12px 5px 8px;
  border-radius: 100px;
  background: color-mix(in srgb, var(--tag, #B0CFFF) 50%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent, #1E62C0) 16%, transparent);
  align-self: flex-start;
  transition: background 0.24s ease, box-shadow 0.24s ease;
}

:global(.group:hover) .score-key-pill {
  background: color-mix(in srgb, var(--tag, #B0CFFF) 75%, transparent);
  box-shadow: 0 2px 10px color-mix(in srgb, var(--accent, #1E62C0) 18%, transparent);
}

.score-key-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent, #1E62C0);
  flex-shrink: 0;
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent, #1E62C0) 20%, transparent);
}

:global(.note-card.is-playing) .score-key-dot {
  animation: dot-pulse 1.6s ease-in-out infinite;
}

@keyframes dot-pulse {
  0%, 100% { box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent, #1E62C0) 20%, transparent); }
  50%       { box-shadow: 0 0 0 5px color-mix(in srgb, var(--accent, #1E62C0) 0%, transparent); }
}

.score-key-label {
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: color-mix(in srgb, var(--accent-dark, #0C3878) 55%, transparent);
}

.score-key-value {
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 13px;
  font-weight: 700;
  font-style: italic;
  color: var(--accent-dark, #0C3878);
  line-height: 1;
}
</style>