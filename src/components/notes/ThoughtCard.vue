<script setup lang="ts">
import type { Note } from '../../types';

defineProps<{
  note: Note;
}>();
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Thought content -->
    <div class="flex-1">
      <p class="text-gray-300 font-serif text-base leading-relaxed italic line-clamp-6">
        "{{ note.content }}"
      </p>
    </div>

    <!-- Metadata -->
    <div v-if="note.metadata?.tags?.length" class="mt-4 pt-4 border-t border-amber-900/20">
      <div class="flex flex-wrap gap-2">
        <span
          v-for="tag in note.metadata.tags.slice(0, 3)"
          :key="tag"
          class="text-[10px] uppercase tracking-wider text-amber-700/60 font-sans"
        >
          #{{ tag }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-6 {
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
