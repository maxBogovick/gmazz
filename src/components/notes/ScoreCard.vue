<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Note } from '../../types';
import { getAssetPath } from '../../api/notes';

const props = defineProps<{
  note: Note;
}>();

defineEmits<{
  click: [note: Note];
}>();

const imagePath = ref<string>('');

onMounted(async () => {
  if (props.note.metadata.file_path) {
    imagePath.value = await getAssetPath(props.note.metadata.file_path);
  }
});

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
      <span class="inline-flex items-center px-2.5 py-1 rounded-md text-xs font-medium bg-blue-500/10 text-blue-400 border border-blue-500/20">
        Score
      </span>
      <time class="text-xs text-[--color-text-muted]">
        {{ formatDate(note.created_at) }}
      </time>
    </div>
    <div class="rounded-lg overflow-hidden border border-[--color-border] bg-[--color-bg-secondary]">
      <img
        v-if="imagePath"
        :src="imagePath"
        alt="Score"
        class="w-full max-h-64 object-contain"
      />
      <div
        v-else
        class="h-32 flex items-center justify-center"
      >
        <svg class="w-8 h-8 text-[--color-text-muted]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      </div>
    </div>
    <p v-if="note.content" class="mt-4 text-sm text-[--color-text-secondary]">
      {{ note.content }}
    </p>
  </article>
</template>
