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
  <div class="w-full space-y-6">
    <div>
      <label class="block text-xs font-medium text-[--color-text-muted] uppercase tracking-wider mb-3">
        Time Signature
      </label>
      <input
        ref="signatureInput"
        type="text"
        :value="timeSignature"
        @input="handleInputSignature"
        @keydown="handleKeydown"
        placeholder="7/4"
        class="w-32 bg-[--color-bg-secondary] border border-[--color-border] rounded-lg px-4 py-3 text-3xl font-[--font-mono] text-[--color-accent-brass] placeholder:text-[--color-text-muted] outline-none focus:border-[--color-border-light] transition-colors"
      />
    </div>

    <div>
      <label class="block text-xs font-medium text-[--color-text-muted] uppercase tracking-wider mb-3">
        Feel / Description
      </label>
      <textarea
        :value="modelValue"
        @input="handleInputContent"
        @keydown="handleKeydown"
        placeholder="Describe the feeling of this rhythm..."
        class="w-full min-h-[120px] bg-[--color-bg-secondary] border border-[--color-border] rounded-lg p-4 text-base leading-relaxed text-[--color-text-primary] font-[--font-serif] italic placeholder:text-[--color-text-muted] placeholder:not-italic resize-none outline-none focus:border-[--color-border-light] transition-colors"
      ></textarea>
    </div>
  </div>
</template>
