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
  const cleaned = target.value.replace(/[^\d/]/g, '');
  emit('update:timeSignature', cleaned);
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
      <label class="block text-[11px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
        Размер
      </label>
      <div class="flex items-center gap-6">
        <div class="bg-white border-2 border-[#D4CAB5] p-6 inline-flex items-center justify-center rounded-lg shadow-sm">
          <input
            ref="signatureInput"
            type="text"
            :value="timeSignature"
            @input="handleInputSignature"
            @keydown="handleKeydown"
            placeholder="7/4"
            class="w-24 bg-transparent border-none text-center text-5xl font-mono font-light text-[#A67C00] placeholder:text-[#8B7E6A] outline-none focus:ring-0"
          />
        </div>
        <div class="text-[#4A3F2F] text-sm font-sans">
          <p class="mb-1">Например:</p>
          <p class="text-[11px] text-[#6B5D4D]">5/4, 7/8, 11/8, 6/4</p>
        </div>
      </div>
    </div>

    <!-- Description Section -->
    <div>
      <label class="block text-[11px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
        Характер / Groove
      </label>
      <textarea
        :value="modelValue"
        @input="handleInputContent"
        @keydown="handleKeydown"
        placeholder="Опишите характер этого ритма..."
        class="w-full min-h-[150px] bg-white border border-[#D4CAB5] rounded-lg p-6 text-lg leading-relaxed text-[#1A1510] font-serif italic placeholder:text-[#8B7E6A] placeholder:not-italic resize-none outline-none focus:border-[#A67C00]/50 transition-colors"
      ></textarea>
    </div>
  </div>
</template>
