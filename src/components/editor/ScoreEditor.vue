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

function clearFile() {
  fileName.value = '';
  previewUrl.value = '';
  isImageFile.value = false;
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
    <!-- Drop Zone -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-amber-700 mb-4 font-sans">
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
          'relative bg-zinc-950 border-2 border-dashed rounded p-8 transition-all duration-300 cursor-pointer',
          isDragging
            ? 'border-amber-500 bg-amber-900/10'
            : 'border-amber-900/30 hover:border-amber-800/50'
        ]"
      >
        <!-- Has Image Preview -->
        <div v-if="previewUrl" class="space-y-4" @click.stop>
          <div class="relative bg-white p-2 rounded shadow-lg">
            <img
              :src="previewUrl"
              :alt="fileName"
              class="max-w-full max-h-[400px] mx-auto rounded"
            />
          </div>
          <div class="flex items-center justify-center gap-4">
            <span class="text-sm text-amber-500 font-mono">{{ fileName }}</span>
            <button
              @click="clearFile"
              class="text-xs text-gray-600 hover:text-amber-500 transition-colors font-sans uppercase tracking-wider"
            >
              Удалить
            </button>
          </div>
        </div>

        <!-- Has PDF (no preview) -->
        <div v-else-if="fileName" class="flex flex-col items-center gap-4 py-8" @click.stop>
          <div class="w-16 h-16 rounded bg-red-500/10 flex items-center justify-center text-red-400">
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
              <path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm4 18H6V4h7v5h5v11z" />
            </svg>
          </div>
          <div class="text-center">
            <p class="text-base text-gray-200">{{ fileName }}</p>
            <p class="text-xs text-gray-600 font-sans uppercase tracking-wider mt-1">PDF документ</p>
          </div>
          <button
            @click="clearFile"
            class="text-xs text-gray-600 hover:text-amber-500 transition-colors font-sans uppercase tracking-wider"
          >
            Удалить
          </button>
        </div>

        <!-- Empty State -->
        <div v-else class="flex flex-col items-center justify-center py-10 text-center">
          <div class="text-5xl text-amber-800/40 mb-4">𝄞</div>
          <p class="text-gray-300 text-lg mb-2">Нажмите или перетащите файл</p>
          <p class="text-xs text-gray-600 font-sans uppercase tracking-wider">PNG, JPG, или PDF</p>
        </div>
      </div>
    </div>

    <!-- Notes -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-amber-700 mb-4 font-sans">
        Аннотация (опционально)
      </label>
      <textarea
        ref="textarea"
        :value="modelValue"
        @input="handleInput"
        @keydown="handleKeydown"
        placeholder="Добавьте описание партитуры..."
        class="w-full min-h-[100px] bg-zinc-950 border border-amber-900/20 rounded p-6 text-base leading-relaxed text-gray-300 placeholder:text-gray-700 resize-none outline-none focus:border-amber-800/40 transition-colors"
      ></textarea>
    </div>
  </div>
</template>
