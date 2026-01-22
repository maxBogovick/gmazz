<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  save: [];
}>();

const textarea = ref<HTMLTextAreaElement | null>(null);

onMounted(() => {
  textarea.value?.focus();
  adjustHeight();
});

function handleInput(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
  adjustHeight();
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault();
    emit('save');
  }
}

function adjustHeight() {
  if (textarea.value) {
    textarea.value.style.height = 'auto';
    textarea.value.style.height = Math.max(200, textarea.value.scrollHeight) + 'px';
  }
}

watch(() => textarea.value, () => {
  adjustHeight();
});
</script>

<template>
  <div class="w-full">
    <label class="block text-[10px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
      Личная Заметка
    </label>
    <textarea
      ref="textarea"
      :value="modelValue"
      @input="handleInput"
      @keydown="handleKeydown"
      placeholder="Начните писать..."
      class="w-full min-h-[250px] bg-transparent border-none p-0 text-xl lg:text-2xl leading-relaxed text-[#3D3428] font-serif italic placeholder:text-[#A89F8B] placeholder:not-italic resize-none outline-none focus:ring-0"
    ></textarea>
  </div>
</template>
