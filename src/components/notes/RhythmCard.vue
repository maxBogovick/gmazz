<script setup lang="ts">
import type { Note } from '../../types';

defineProps<{
  note: Note;
}>();

defineEmits<{
  click: [note: Note];
}>();

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}
</script>

<template>
  <article
    class="group bg-[--color-bg-card] border border-[--color-border] rounded-lg p-6 cursor-pointer hover:border-[--color-border-light] hover:bg-[--color-bg-tertiary] transition-all"
    @click="$emit('click', note)"
  >
    <div class="flex items-start justify-between gap-4 mb-4">
      <span class="inline-flex items-center px-2.5 py-1 rounded-md text-xs font-medium bg-purple-500/10 text-purple-400 border border-purple-500/20">
        Rhythm
      </span>
      <time class="text-xs text-[--color-text-muted]">
        {{ formatDate(note.created_at) }}
      </time>
    </div>
    <div class="flex items-baseline gap-4">
      <span
        v-if="note.metadata.time_signature"
        class="text-3xl font-[--font-mono] font-medium text-[--color-accent-brass]"
      >
        {{ note.metadata.time_signature }}
      </span>
      <p class="text-base text-[--color-text-secondary] italic">
        {{ note.content }}
      </p>
    </div>
  </article>
</template>
