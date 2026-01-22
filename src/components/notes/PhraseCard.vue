<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import type { Note } from '../../types';
import { getAssetPath } from '../../api/notes';
import Waveform from '../common/Waveform.vue';

const props = defineProps<{
  note: Note;
}>();

defineEmits<{
  click: [note: Note];
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
      <span class="inline-flex items-center px-2.5 py-1 rounded-md text-xs font-medium bg-green-500/10 text-green-400 border border-green-500/20">
        Phrase
      </span>
      <time class="text-xs text-[--color-text-muted]">
        {{ formatDate(note.created_at) }}
      </time>
    </div>
    <div class="flex items-center gap-4 p-4 bg-[--color-bg-secondary] rounded-lg border border-[--color-border]">
      <button
        @click="togglePlay"
        class="w-12 h-12 flex items-center justify-center rounded-full bg-[--color-accent-brass] hover:bg-[--color-accent-amber] text-[--color-bg-primary] transition-colors flex-shrink-0"
        :aria-label="isPlaying ? 'Pause' : 'Play'"
      >
        <svg v-if="!isPlaying" class="w-5 h-5 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M8 5v14l11-7z" />
        </svg>
        <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
        </svg>
      </button>
      <Waveform class="flex-1 h-12" />
    </div>
    <p v-if="note.metadata.comment || note.content" class="mt-4 text-sm text-[--color-text-secondary]">
      {{ note.metadata.comment || note.content }}
    </p>
  </article>
</template>
