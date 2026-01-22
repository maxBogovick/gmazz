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
      <span class="inline-flex items-center px-2.5 py-1 rounded-md text-xs font-medium bg-[--color-accent-dim] text-[--color-accent-brass] border border-[--color-accent-brass]/20">
        Harmony
      </span>
      <time class="text-xs text-[--color-text-muted]">
        {{ formatDate(note.created_at) }}
      </time>
    </div>
    <pre class="font-[--font-mono] text-sm text-[--color-text-primary] whitespace-pre-wrap leading-relaxed bg-[--color-bg-secondary] rounded-md p-4 border border-[--color-border]">{{ note.content }}</pre>
  </article>
</template>
