<script setup lang="ts">
import type { Note } from '../../types';

defineProps<{
  note: Note;
}>();
</script>

<template>
  <div class="thought-root h-full flex flex-col gap-3">

    <!-- ── Quote body ─────────────────────────────── -->
    <div class="thought-quote-wrap flex-1">
      <!-- Decorative open quote -->
      <span class="thought-open-quote" aria-hidden="true">"</span>

      <p class="thought-text">{{ note.content }}</p>

      <!-- Closing quote — floats to the right -->
      <span class="thought-close-quote" aria-hidden="true">"</span>
    </div>

    <!-- ── Tags ───────────────────────────────────── -->
    <div v-if="note.metadata?.tags?.length" class="thought-tags">
      <span
          v-for="tag in note.metadata.tags.slice(0, 3)"
          :key="tag"
          class="thought-tag"
      >#{{ tag }}</span>
    </div>

  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,600;0,700;1,400;1,600&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap');

/* ───────────────────────────────────────────────────────
   QUOTE WRAPPER
─────────────────────────────────────────────────────── */
.thought-quote-wrap {
  position: relative;
  padding: 4px 6px 4px 22px;
}

/* Vertical accent rule */
.thought-quote-wrap::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 2px;
  border-radius: 2px;
  background: linear-gradient(
      180deg,
      var(--accent, #D4760A) 0%,
      color-mix(in srgb, var(--accent, #D4760A) 25%, transparent) 100%
  );
  opacity: 0.55;
  transition: opacity 0.28s ease;
}

:global(.group:hover) .thought-quote-wrap::before {
  opacity: 1;
}

/* ── Decorative quotes ───────────────────────────────── */
.thought-open-quote,
.thought-close-quote {
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 64px;
  line-height: 0;
  color: color-mix(in srgb, var(--tag, #FDE89A) 90%, var(--accent, #D4760A));
  pointer-events: none;
  user-select: none;
  transition: color 0.3s ease;
}

.thought-open-quote {
  position: absolute;
  top: 18px;
  left: 6px;
  opacity: 0.7;
}

.thought-close-quote {
  float: right;
  position: relative;
  top: -8px;
  margin-left: 4px;
  opacity: 0.45;
}

:global(.group:hover) .thought-open-quote,
:global(.group:hover) .thought-close-quote {
  color: color-mix(in srgb, var(--accent, #D4760A) 40%, var(--tag, #FDE89A));
  opacity: 0.9;
}

/* ── Quote text ──────────────────────────────────────── */
.thought-text {
  font-family: 'Cormorant Garamond', Georgia, serif;
  font-size: 15px;
  font-weight: 600;
  font-style: italic;
  line-height: 1.72;
  color: color-mix(in srgb, var(--accent-dark, #8B4A00) 85%, #1a1510);
  letter-spacing: 0.012em;

  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;

  transition: color 0.28s ease;
}

:global(.group:hover) .thought-text {
  color: var(--accent-dark, #8B4A00);
}

/* ───────────────────────────────────────────────────────
   TAGS
─────────────────────────────────────────────────────── */
.thought-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding-top: 10px;
  border-top: 1px solid color-mix(in srgb, var(--accent, #D4760A) 12%, transparent);
}

.thought-tag {
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--tag-text, #7A4400);
  padding: 3px 10px;
  border-radius: 100px;
  background: color-mix(in srgb, var(--tag, #FDE89A) 60%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent, #D4760A) 16%, transparent);
  transition: background 0.22s ease, box-shadow 0.22s ease;
}

:global(.group:hover) .thought-tag {
  background: color-mix(in srgb, var(--tag, #FDE89A) 85%, transparent);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent, #D4760A) 18%, transparent);
}
</style>