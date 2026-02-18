<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import type { Note } from '../../types';
import { getAssetPath } from '../../api/notes';

const props = defineProps<{
  note: Note;
}>();

const audioPath = ref<string>('');
const isPlaying = ref(false);
const audio = ref<HTMLAudioElement | null>(null);
const bars = ref<number[]>([]);
let audioEndedHandler: (() => void) | null = null;
let lastObjectUrl: string | null = null;

onMounted(async () => {
  bars.value = Array.from({ length: 20 }, () => 8 + Math.random() * 16);
  if (props.note.metadata.file_path) {
    const path = await getAssetPath(props.note.metadata.file_path);
    if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
      URL.revokeObjectURL(lastObjectUrl);
    }
    audioPath.value = path;
    lastObjectUrl = path;
    audio.value = new Audio(audioPath.value);
    audioEndedHandler = () => {
      isPlaying.value = false;
    };
    audio.value.addEventListener('ended', audioEndedHandler);
  }
});

onUnmounted(() => {
  if (audio.value) {
    if (audioEndedHandler) {
      audio.value.removeEventListener('ended', audioEndedHandler);
    }
    audio.value.pause();
    audio.value = null;
  }
  if (lastObjectUrl && lastObjectUrl.startsWith('blob:')) {
    URL.revokeObjectURL(lastObjectUrl);
  }
});

function togglePlay(event: Event) {
  event.stopPropagation();
  if (!audio.value) return;

  if (isPlaying.value) {
    audio.value.pause();
  } else {
    audio.value.play();
  }
  isPlaying.value = !isPlaying.value;
}
</script>

<template>
  <div class="h-full flex flex-col">

    <!-- Comment -->
    <div v-if="note.content" class="mt-4 pt-3 border-t border-[#E0D9C8]">
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
