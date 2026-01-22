<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Note } from '../../types';
import { getAssetPath } from '../../api/notes';

const props = defineProps<{
  note: Note;
}>();

const imagePath = ref<string>('');

onMounted(async () => {
  if (props.note.metadata.file_path) {
    imagePath.value = await getAssetPath(props.note.metadata.file_path);
  }
});
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Score Preview -->
    <div class="flex-1 flex items-center justify-center min-h-[160px] bg-[#F5F1E8] rounded-lg border border-[#E0D9C8] overflow-hidden">
      <img
        v-if="imagePath"
        :src="imagePath"
        alt="Score"
        class="w-full h-full object-contain"
      />
      <div v-else class="text-center p-4">
        <div class="text-5xl text-[#A67C00]/40 mb-2">𝄞</div>
        <div class="text-xs text-[#8B7E6A] font-sans uppercase tracking-wider">
          {{ note.metadata.file_path || 'Score' }}
        </div>
      </div>
    </div>

    <!-- Caption -->
    <div v-if="note.content" class="mt-4">
      <p class="text-sm text-[#5C5245] font-light line-clamp-2">
        {{ note.content }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
