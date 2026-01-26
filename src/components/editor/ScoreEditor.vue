<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

defineProps<{
  modelValue: string;
  filePath?: string;
  audioPath?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:filePath': [path: string, data: number[]];
  'update:audioPath': [path: string, data: number[]];
  save: [];
}>();

const textarea = ref<HTMLTextAreaElement | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const audioInput = ref<HTMLInputElement | null>(null);
const isDragging = ref(false);
const fileName = ref<string>('');
const audioFileName = ref<string>('');
const previewUrl = ref<string>('');
const isImageFile = ref(false);

onMounted(() => {
  document.addEventListener('dragover', handleDragOver);
  document.addEventListener('dragleave', handleDragLeave);
  document.addEventListener('drop', handleDrop);
});

onUnmounted(() => {
  document.removeEventListener('dragover', handleDragOver);
  document.removeEventListener('dragleave', handleDragLeave);
  document.removeEventListener('drop', handleDrop);
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
  }
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

  // Simple heuristic: if audio, treat as audio path; else score path
  const file = files[0];
  if (file.type.startsWith('audio/')) {
      processAudio(file);
  } else {
      processFile(file);
  }
}

function triggerFileDialog() {
  fileInput.value?.click();
}

function triggerAudioDialog() {
  audioInput.value?.click();
}

async function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    processFile(target.files[0]);
  }
}

async function handleAudioSelect(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    processAudio(target.files[0]);
  }
}

async function processFile(file: File) {
  const isImage = file.type.startsWith('image/');
  const isPdf = file.type === 'application/pdf';

  if (!isImage && !isPdf) {
    alert('Пожалуйста, выберите изображение или PDF файл.');
    return;
  }

  fileName.value = file.name;
  isImageFile.value = isImage;

  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
  }

  if (isImage) {
    previewUrl.value = URL.createObjectURL(file);
  } else {
    previewUrl.value = '';
  }

  const arrayBuffer = await file.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);
  emit('update:filePath', file.name, Array.from(uint8Array));

  textarea.value?.focus();
}

async function processAudio(file: File) {
  if (!file.type.startsWith('audio/')) {
    alert('Пожалуйста, выберите аудио файл.');
    return;
  }

  audioFileName.value = file.name;
  
  const arrayBuffer = await file.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);
  emit('update:audioPath', file.name, Array.from(uint8Array));
}

function clearFile() {
  fileName.value = '';
  previewUrl.value = '';
  isImageFile.value = false;
  emit('update:filePath', '', []);
}

function clearAudio() {
  audioFileName.value = '';
  emit('update:audioPath', '', []);
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
    <!-- Drop Zone (Score) -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
        Файл Партитуры
      </label>

      <!-- Hidden File Input -->
      <input
        type="file"
        ref="fileInput"
        class="hidden"
        accept="image/*,application/pdf"
        @change="handleFileSelect"
      />

      <div
        @click="triggerFileDialog"
        :class="[
          'relative bg-[#F5F1E8] border-2 border-dashed rounded-lg p-8 transition-all duration-300 cursor-pointer',
          isDragging
            ? 'border-[#A67C00] bg-[#A67C00]/5'
            : 'border-[#D4CAB5] hover:border-[#A67C00]/50'
        ]"
      >
        <!-- Has Image Preview -->
        <div v-if="previewUrl" class="space-y-4" @click.stop>
          <div class="relative bg-white p-3 rounded-lg shadow-md border border-[#E0D9C8]">
            <img
              :src="previewUrl"
              :alt="fileName"
              class="max-w-full max-h-[400px] mx-auto rounded"
            />
          </div>
          <div class="flex items-center justify-center gap-4">
            <span class="text-sm text-[#A67C00] font-mono">{{ fileName }}</span>
            <button
              @click="clearFile"
              class="text-xs text-[#4A3F2F] hover:text-[#A67C00] transition-colors font-sans uppercase tracking-wider"
            >
              Удалить
            </button>
          </div>
        </div>

        <!-- Has PDF (no preview) -->
        <div v-else-if="fileName" class="flex flex-col items-center gap-4 py-8" @click.stop>
          <div class="w-16 h-16 rounded-lg bg-[#A67C00]/10 flex items-center justify-center text-[#A67C00]">
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
              <path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z" />
            </svg>
          </div>
          <div class="text-center">
            <p class="text-base text-[#1A1510]">{{ fileName }}</p>
            <p class="text-xs text-[#4A3F2F] font-sans uppercase tracking-wider mt-1">PDF документ</p>
          </div>
          <button
            @click="clearFile"
            class="text-xs text-[#4A3F2F] hover:text-[#A67C00] transition-colors font-sans uppercase tracking-wider"
          >
            Удалить
          </button>
        </div>

        <!-- Empty State -->
        <div v-else class="flex flex-col items-center justify-center py-10 text-center">
          <div class="text-5xl text-[#A67C00]/40 mb-4">𝄞</div>
          <p class="text-[#1A1510] text-lg mb-2">Нажмите или перетащите файл</p>
          <p class="text-xs text-[#4A3F2F] font-sans uppercase tracking-wider">PNG, JPG, или PDF</p>
        </div>
      </div>
    </div>

    <!-- Drop Zone (Audio) -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
        Аудиозапись (опционально)
      </label>

      <!-- Hidden Audio Input -->
      <input
        type="file"
        ref="audioInput"
        class="hidden"
        accept="audio/*"
        @change="handleAudioSelect"
      />

      <div
        @click="triggerAudioDialog"
        class="relative bg-[#F5F1E8] border-2 border-dashed border-[#D4CAB5] hover:border-[#A67C00]/50 rounded-lg p-6 transition-all duration-300 cursor-pointer flex items-center justify-center"
      >
        <div v-if="audioFileName" class="flex items-center gap-4 w-full justify-between px-4" @click.stop>
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-[#A67C00]/10 flex items-center justify-center text-[#A67C00]">
              ▶
            </div>
            <span class="text-sm text-[#1A1510] font-mono truncate max-w-[200px]">{{ audioFileName }}</span>
          </div>
          <button
            @click="clearAudio"
            class="text-xs text-[#4A3F2F] hover:text-[#A67C00] transition-colors font-sans uppercase tracking-wider"
          >
            Удалить
          </button>
        </div>

        <div v-else class="text-center py-4">
          <p class="text-sm text-[#8B7E6A] flex items-center gap-2">
            <span class="text-xl">🎙</span> Добавить аудио
          </p>
        </div>
      </div>
    </div>

    <!-- Notes -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
        Аннотация (опционально)
      </label>
      <textarea
        ref="textarea"
        :value="modelValue"
        @input="handleInput"
        @keydown="handleKeydown"
        placeholder="Добавьте описание партитуры..."
        class="w-full min-h-[100px] bg-white border border-[#D4CAB5] rounded-lg p-6 text-base leading-relaxed text-[#1A1510] placeholder:text-[#8B7E6A] resize-none outline-none focus:border-[#A67C00]/50 transition-colors"
      ></textarea>
    </div>
  </div>
</template>
