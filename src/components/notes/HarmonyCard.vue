<script setup lang="ts">
import type { Note } from '../../types';

defineProps<{
  note: Note;
}>();
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Chord Symbol -->
    <div v-if="note.metadata?.chord_symbol" class="mb-4">
      <div class="text-4xl font-mono text-[#6B4423] tracking-wide">
        {{ note.metadata.chord_symbol }}
      </div>
      <div v-if="note.metadata?.key" class="text-[11px] text-[#4A3F2F] mt-1 font-sans uppercase tracking-wider">
        Key of {{ note.metadata.key }}
      </div>
    </div>

    <!-- Voicing/Content -->
    <div class="flex-1">
      <pre class="font-mono text-sm text-[#2C2416] whitespace-pre-wrap leading-relaxed line-clamp-5">{{ note.content }}</pre>
    </div>

    <!-- Voice Leading indicator -->
    <div v-if="note.metadata?.voice_leading" class="mt-4 pt-3 border-t border-[#E0D9C8]">
      <div class="text-[11px] uppercase tracking-wider text-[#A67C00] font-sans">
        Voice Leading
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-5 {
  display: -webkit-box;
  -webkit-line-clamp: 5;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
