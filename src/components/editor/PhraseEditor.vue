<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

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
const fileInput = ref<HTMLInputElement | null>(null);
const isDragging = ref(false);
const fileName = ref<string>('');
const isRecording = ref(false);
const recordingTime = ref(0);
const mediaRecorder = ref<MediaRecorder | null>(null);
const audioChunks = ref<Blob[]>([]);
const recordingTimer = ref<number | null>(null);
const playbackUrl = ref<string | null>(null);
const recordingMimeType = ref<string>('');

function getSupportedMimeType() {
  const types = [
    'audio/webm;codecs=opus',
    'audio/webm',
    'audio/mp4',
    'audio/aac',
    'audio/ogg'
  ];
  for (const type of types) {
    if (MediaRecorder.isTypeSupported(type)) {
      return type;
    }
  }
  return '';
}

onMounted(() => {
  document.addEventListener('dragover', handleDragOver);
  document.addEventListener('dragleave', handleDragLeave);
  document.addEventListener('drop', handleDrop);
});

onUnmounted(() => {
  document.removeEventListener('dragover', handleDragOver);
  document.removeEventListener('dragleave', handleDragLeave);
  document.removeEventListener('drop', handleDrop);
  if (recordingTimer.value) clearInterval(recordingTimer.value);
  if (playbackUrl.value) URL.revokeObjectURL(playbackUrl.value);
  stopRecording();
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

  processFile(files[0]);
}

function triggerFileDialog() {
  fileInput.value?.click();
}

async function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    processFile(target.files[0]);
  }
}

async function processFile(file: File) {
  if (!file.type.startsWith('audio/')) {
    alert('Пожалуйста, выберите аудио файл.');
    return;
  }

  fileName.value = file.name;

  if (playbackUrl.value) URL.revokeObjectURL(playbackUrl.value);
  playbackUrl.value = URL.createObjectURL(file);

  const arrayBuffer = await file.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);
  emit('update:filePath', file.name, Array.from(uint8Array));

  textarea.value?.focus();
}

async function toggleRecording(e: Event) {
  e.stopPropagation();
  if (isRecording.value) {
    stopRecording();
  } else {
    await startRecording();
  }
}

async function startRecording() {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

    const mimeType = getSupportedMimeType();
    recordingMimeType.value = mimeType;

    const options = mimeType ? { mimeType } : undefined;
    mediaRecorder.value = new MediaRecorder(stream, options);
    audioChunks.value = [];

    mediaRecorder.value.ondataavailable = (event) => {
      audioChunks.value.push(event.data);
    };

    mediaRecorder.value.onstop = async () => {
      const type = recordingMimeType.value || 'audio/webm';
      const ext = type.includes('mp4') || type.includes('aac') ? 'm4a' : 'webm';

      const audioBlob = new Blob(audioChunks.value, { type });
      const file = new File([audioBlob], `recording_${new Date().getTime()}.${ext}`, { type });
      await processFile(file);
    };

    mediaRecorder.value.start();
    isRecording.value = true;
    recordingTime.value = 0;

    recordingTimer.value = window.setInterval(() => {
      recordingTime.value++;
    }, 1000);

  } catch (err) {
    console.error('Error accessing microphone:', err);
    alert('Доступ к микрофону запрещён. Проверьте системные настройки.');
  }
}

function stopRecording() {
  if (mediaRecorder.value && isRecording.value) {
    mediaRecorder.value.stop();
    mediaRecorder.value.stream.getTracks().forEach(track => track.stop());
    isRecording.value = false;
    if (recordingTimer.value) {
      clearInterval(recordingTimer.value);
      recordingTimer.value = null;
    }
  }
}

function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function clearRecording() {
  fileName.value = '';
  playbackUrl.value = null;
  emit('update:filePath', '', []);
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
  <div class="w-full space-y-8">
    <!-- Recording / File Area -->
    <div>
      <div class="flex items-center justify-between mb-4">
        <label class="text-[10px] uppercase tracking-[0.2em] text-amber-700 font-sans">
          Аудио Источник
        </label>
        <span v-if="isRecording" class="text-xs font-mono text-red-500 animate-pulse flex items-center gap-2">
          <span class="w-2 h-2 bg-red-500 rounded-full"></span>
          Запись {{ formatTime(recordingTime) }}
        </span>
      </div>

      <!-- Hidden File Input -->
      <input
        type="file"
        ref="fileInput"
        class="hidden"
        accept="audio/*"
        @change="handleFileSelect"
      />

      <div
        @click="triggerFileDialog"
        :class="[
          'relative bg-zinc-950 border-2 border-dashed rounded p-8 transition-all duration-300 cursor-pointer',
          isDragging
            ? 'border-amber-500 bg-amber-900/10'
            : isRecording
              ? 'border-red-500/50 bg-red-500/5'
              : 'border-amber-900/30 hover:border-amber-800/50'
        ]"
      >
        <!-- Has Audio -->
        <div v-if="fileName || playbackUrl" class="flex flex-col items-center gap-6" @click.stop>
          <!-- Audio Player -->
          <div class="w-full bg-black border border-amber-900/20 p-4 rounded">
            <audio :src="playbackUrl" controls class="w-full h-8" />
          </div>

          <div class="flex items-center gap-3 text-amber-500">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="text-sm font-mono truncate max-w-[200px]">{{ fileName }}</span>
          </div>

          <button
            @click="clearRecording"
            class="text-xs text-gray-600 hover:text-amber-500 transition-colors font-sans uppercase tracking-wider"
          >
            Очистить и записать новое
          </button>
        </div>

        <!-- Empty State: Record or Drop -->
        <div v-else class="flex flex-col items-center justify-center py-4">
          <!-- Record Button -->
          <button
            @click="toggleRecording"
            class="w-20 h-20 rounded-full flex items-center justify-center transition-all duration-300 mb-6 relative"
            :class="isRecording
              ? 'bg-red-500 shadow-[0_0_30px_rgba(239,68,68,0.4)] scale-110'
              : 'bg-gradient-to-br from-amber-600 to-amber-700 hover:from-amber-500 hover:to-amber-600 shadow-[0_0_20px_rgba(212,175,55,0.2)]'"
          >
            <svg v-if="!isRecording" class="w-8 h-8 text-black" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
            </svg>
            <div v-else class="w-8 h-8 bg-white rounded"></div>
          </button>

          <div v-if="!isRecording" class="text-center space-y-2">
            <p class="text-gray-300 text-lg">Нажмите для записи</p>
            <p class="text-gray-600 text-xs font-sans uppercase tracking-wider">или перетащите аудио файл</p>
          </div>
          <div v-else class="text-center">
            <p class="text-red-400 font-mono text-sm uppercase tracking-wider">Идёт запись...</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Comment -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-amber-700 mb-4 font-sans">
        Комментарий (опционально)
      </label>
      <textarea
        ref="textarea"
        :value="modelValue"
        @input="handleInput"
        @keydown="handleKeydown"
        placeholder="Опишите настроение, тональность или контекст..."
        class="w-full min-h-[120px] bg-zinc-950 border border-amber-900/20 rounded p-6 text-base leading-relaxed text-gray-300 font-serif italic placeholder:text-gray-700 placeholder:not-italic resize-none outline-none focus:border-amber-800/40 transition-colors"
      ></textarea>
    </div>
  </div>
</template>
