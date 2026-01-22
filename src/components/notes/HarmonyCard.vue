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
      <div class="text-4xl font-mono text-[#8B5A2B] tracking-wide">
        {{ note.metadata.chord_symbol }}
      </div>
      <div v-if="note.metadata?.key" class="text-xs text-[#8B7E6A] mt-1 font-sans uppercase tracking-wider">
        Key of {{ note.metadata.key }}
      </div>
    </div>

    <!-- Voicing/Content -->
    <div class="flex-1">
      <pre class="font-mono text-sm text-[#5C5245] whitespace-pre-wrap leading-relaxed line-clamp-5">{{ note.content }}</pre>
    </div>

    <!-- Voice Leading indicator -->
    <div v-if="note.metadata?.voice_leading" class="mt-4 pt-3 border-t border-[#E0D9C8]">
      <div class="text-[10px] uppercase tracking-wider text-[#A67C00] font-sans">
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
