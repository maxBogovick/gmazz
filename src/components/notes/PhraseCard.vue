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

onMounted(async () => {
  if (props.note.metadata.file_path) {
    audioPath.value = await getAssetPath(props.note.metadata.file_path);
    audio.value = new Audio(audioPath.value);
    audio.value.addEventListener('ended', () => {
      isPlaying.value = false;
    });
  }
});

onUnmounted(() => {
  if (audio.value) {
    audio.value.pause();
    audio.value = null;
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
    <!-- Audio Player -->
    <div class="flex-1 flex flex-col items-center justify-center min-h-[140px]">
      <!-- Play Button -->
      <button
        v-if="audioPath"
        @click="togglePlay"
        class="w-16 h-16 flex items-center justify-center rounded-full border-2 border-[#D4CAB5] text-[#A67C00] hover:bg-[#A67C00]/10 hover:border-[#A67C00] transition-all duration-300"
        :aria-label="isPlaying ? 'Pause' : 'Play'"
      >
        <svg v-if="!isPlaying" class="w-6 h-6 ml-1" fill="currentColor" viewBox="0 0 24 24">
          <path d="M8 5v14l11-7z" />
        </svg>
        <svg v-else class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
        </svg>
      </button>

      <!-- No Audio Placeholder -->
      <div v-else class="text-center">
        <div class="text-4xl text-[#A67C00]/40 mb-2">♫</div>
        <div class="text-xs text-[#8B7E6A] font-sans uppercase tracking-wider">Audio Phrase</div>
      </div>

      <!-- Waveform Visual -->
      <div v-if="audioPath" class="mt-4 flex items-center gap-1 h-8">
        <div
          v-for="i in 20"
          :key="i"
          class="w-1 bg-[#A67C00]/30 rounded-full transition-all duration-300"
          :class="isPlaying ? 'animate-pulse' : ''"
          :style="{ height: `${8 + Math.random() * 16}px` }"
        ></div>
      </div>
    </div>

    <!-- Comment -->
    <div v-if="note.content" class="mt-4 pt-3 border-t border-[#E0D9C8]">
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
