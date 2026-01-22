<script setup lang="ts">
import { ref, onMounted } from 'vue';

defineProps<{
  modelValue: string;
  timeSignature?: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:timeSignature': [value: string];
  save: [];
}>();

const signatureInput = ref<HTMLInputElement | null>(null);

onMounted(() => {
  signatureInput.value?.focus();
});

function handleInputContent(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
}

function handleInputSignature(event: Event) {
  const target = event.target as HTMLInputElement;
  emit('update:timeSignature', target.value);
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault();
    emit('save');
  }
}
</script>

<template>
  <div class="w-full space-y-10">
    <!-- Time Signature Section -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-amber-700 mb-4 font-sans">
        Размер
      </label>
      <div class="flex items-center gap-6">
        <div class="bg-zinc-950 border-2 border-amber-900/40 p-6 inline-flex items-center justify-center">
          <input
            ref="signatureInput"
            type="text"
            :value="timeSignature"
            @input="handleInputSignature"
            @keydown="handleKeydown"
            placeholder="7/4"
            class="w-24 bg-transparent border-none text-center text-5xl font-mono font-light text-amber-400 placeholder:text-gray-700 outline-none focus:ring-0"
          />
        </div>
        <div class="text-gray-600 text-sm font-sans">
          <p class="mb-1">Например:</p>
          <p class="text-xs text-gray-700">5/4, 7/8, 11/8, 6/4</p>
        </div>
      </div>
    </div>

    <!-- Description Section -->
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-amber-700 mb-4 font-sans">
        Характер / Groove
      </label>
      <textarea
        :value="modelValue"
        @input="handleInputContent"
        @keydown="handleKeydown"
        placeholder="Опишите характер этого ритма..."
        class="w-full min-h-[150px] bg-zinc-950 border border-amber-900/20 rounded p-6 text-lg leading-relaxed text-gray-300 font-serif italic placeholder:text-gray-700 placeholder:not-italic resize-none outline-none focus:border-amber-800/40 transition-colors"
      ></textarea>
    </div>
  </div>
</template>
