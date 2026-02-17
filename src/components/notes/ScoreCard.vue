<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import type { Note } from '../../types';
import { getAssetPath } from '../../api/notes';

const props = defineProps<{
  note: Note;
}>();

const imagePath = ref<string>('');
let lastObjectUrl: string | null = null;

async function loadImage() {
  if (props.note.metadata.file_path) {
    const path = await getAssetPath(props.note.metadata.file_path);
    if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
      URL.revokeObjectURL(lastObjectUrl);
    }
    imagePath.value = path;
    lastObjectUrl = path;
  } else {
    if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
      URL.revokeObjectURL(lastObjectUrl);
    }
    lastObjectUrl = null;
    imagePath.value = '';
  }
}

onMounted(loadImage);

watch(() => props.note.metadata.file_path, () => {
  loadImage();
});

onUnmounted(() => {
  if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
    URL.revokeObjectURL(lastObjectUrl);
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
        <div class="text-[11px] text-[#4A3F2F] font-sans uppercase tracking-wider">
          {{ note.metadata.file_path || 'Score' }}
        </div>
      </div>
    </div>

    <!-- Caption -->
    <div v-if="note.content" class="mt-4">
      <p class="text-sm text-[#2C2416] font-light line-clamp-2">
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
