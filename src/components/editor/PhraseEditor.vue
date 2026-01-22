<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import Waveform from '../common/Waveform.vue';

defineProps<{
  modelValue: string;
  filePath?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:filePath': [path: string, data: number[]];
  save: [];
}>();

const textarea = ref<HTMLTextAreaElement | null>(null);
const isDragging = ref(false);
const fileName = ref<string>('');

onMounted(() => {
  document.addEventListener('dragover', handleDragOver);
  document.addEventListener('dragleave', handleDragLeave);
  document.addEventListener('drop', handleDrop);
});

onUnmounted(() => {
  document.removeEventListener('dragover', handleDragOver);
  document.removeEventListener('dragleave', handleDragLeave);
  document.removeEventListener('drop', handleDrop);
});

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  isDragging.value = true;
}

function handleDragLeave(event: DragEvent) {
  event.preventDefault();
  isDragging.value = false;
}

async function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragging.value = false;

  const files = event.dataTransfer?.files;
  if (!files || files.length === 0) return;

  const file = files[0];
  if (!file.type.startsWith('audio/')) return;

  fileName.value = file.name;

  const arrayBuffer = await file.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);
  emit('update:filePath', file.name, Array.from(uint8Array));

  textarea.value?.focus();
}

function handleInput(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault();
    emit('save');
  }
}
</script>

<template>
  <div class="w-full space-y-6">
    <div>
      <label class="block text-xs font-medium text-[--color-text-muted] uppercase tracking-wider mb-3">
        Audio File
      </label>
      <div
        :class="[
          'p-6 border-2 border-dashed rounded-lg transition-all',
          isDragging
            ? 'border-[--color-accent-brass] bg-[--color-accent-dim]'
            : 'border-[--color-border] hover:border-[--color-border-light]'
        ]"
      >
        <div v-if="fileName || filePath" class="flex items-center gap-4">
          <div class="w-12 h-12 rounded-full bg-[--color-accent-brass] flex items-center justify-center text-[--color-bg-primary]">
            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-[--color-text-primary] truncate">{{ fileName || 'Audio file' }}</p>
            <Waveform class="mt-2 h-10" />
          </div>
        </div>
        <div v-else class="text-center py-4">
          <svg class="w-10 h-10 mx-auto text-[--color-text-muted] mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          <p class="text-sm text-[--color-text-secondary]">Drop audio file here</p>
          <p class="text-xs text-[--color-text-muted] mt-1">MP3, WAV, or other audio formats</p>
        </div>
      </div>
    </div>

    <div>
      <label class="block text-xs font-medium text-[--color-text-muted] uppercase tracking-wider mb-3">
        Comment (optional)
      </label>
      <textarea
        ref="textarea"
        :value="modelValue"
        @input="handleInput"
        @keydown="handleKeydown"
        placeholder="Add a note about this phrase..."
        class="w-full min-h-[100px] bg-[--color-bg-secondary] border border-[--color-border] rounded-lg p-4 text-base leading-relaxed text-[--color-text-primary] placeholder:text-[--color-text-muted] resize-none outline-none focus:border-[--color-border-light] transition-colors"
      ></textarea>
    </div>
  </div>
</template>
