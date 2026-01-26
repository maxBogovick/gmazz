<script setup lang="ts">
import type { Note } from '../../types';

defineProps<{
  note: Note;
}>();
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Time Signature Display -->
    <div class="flex-1 flex flex-col items-center justify-center">
      <div
        v-if="note.metadata?.time_signature"
        class="text-center"
      >
        <!-- Time Signature -->
        <div class="text-6xl font-mono font-light text-[#6B4423] tracking-wider mb-2">
          {{ note.metadata.time_signature }}
        </div>
        <div class="text-[11px] uppercase tracking-[0.2em] text-[#4A3F2F] font-sans">
          Time Signature
        </div>
      </div>

      <div v-else class="text-center">
        <div class="text-5xl text-[#A67C00]/40 mb-2">𝅘𝅥𝅮</div>
        <div class="text-[11px] text-[#4A3F2F] font-sans uppercase tracking-wider">
          Rhythm Pattern
        </div>
      </div>
    </div>

    <!-- Feel/Description -->
    <div v-if="note.content" class="mt-4 pt-4 border-t border-[#E0D9C8]">
      <p class="text-sm text-[#2C2416] font-serif italic text-center line-clamp-3">
        {{ note.content }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
