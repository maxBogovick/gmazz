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
    <label class="block text-xs font-medium text-[--color-text-muted] uppercase tracking-wider mb-3">
      Your thought
    </label>
    <textarea
      ref="textarea"
      :value="modelValue"
      @input="handleInput"
      @keydown="handleKeydown"
      placeholder="Write your thought, idea, or reflection..."
      class="w-full min-h-[200px] bg-[--color-bg-secondary] border border-[--color-border] rounded-lg p-4 text-base leading-relaxed text-[--color-text-primary] font-[--font-serif] placeholder:text-[--color-text-muted] resize-none outline-none focus:border-[--color-border-light] transition-colors"
    ></textarea>
  </div>
</template>
