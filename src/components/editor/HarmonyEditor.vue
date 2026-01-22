<script setup lang="ts">
import { ref, onMounted } from 'vue';

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  save: [];
}>();

const textarea = ref<HTMLTextAreaElement | null>(null);

onMounted(() => {
  textarea.value?.focus();
});

function handleInput(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
    event.preventDefault();
    emit('save');
  }
  if (event.key === 'Tab') {
    event.preventDefault();
    const target = event.target as HTMLTextAreaElement;
    const start = target.selectionStart;
    const end = target.selectionEnd;
    const spaces = '    ';
    const newValue = props.modelValue.substring(0, start) + spaces + props.modelValue.substring(end);
    emit('update:modelValue', newValue);
    setTimeout(() => {
      target.selectionStart = target.selectionEnd = start + spaces.length;
    }, 0);
  }
}
</script>

<template>
  <div class="w-full">
    <label class="block text-xs font-medium text-[--color-text-muted] uppercase tracking-wider mb-3">
      Chord progression
    </label>
    <textarea
      ref="textarea"
      :value="modelValue"
      @input="handleInput"
      @keydown="handleKeydown"
      placeholder="Cmaj7  |  Dm7  |  G7  |  Cmaj7"
      class="w-full min-h-[200px] bg-[--color-bg-secondary] border border-[--color-border] rounded-lg p-4 text-sm leading-relaxed text-[--color-text-primary] font-[--font-mono] placeholder:text-[--color-text-muted] resize-none outline-none focus:border-[--color-border-light] transition-colors whitespace-pre"
    ></textarea>
    <p class="text-xs text-[--color-text-muted] mt-2">
      Press Tab to align chords
    </p>
  </div>
</template>
