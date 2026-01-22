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
  <div class="w-full space-y-6">
    <div>
      <label class="block text-[10px] uppercase tracking-[0.2em] text-[#A67C00] mb-4 font-sans">
        Гармоническая Последовательность
      </label>
      <div class="bg-white border border-[#D4CAB5] p-6 rounded-lg shadow-sm">
        <textarea
          ref="textarea"
          :value="modelValue"
          @input="handleInput"
          @keydown="handleKeydown"
          placeholder="Cmaj7  |  Dm7  |  G7  |  Cmaj7"
          class="w-full min-h-[200px] bg-transparent border-none p-0 text-lg leading-loose text-[#A67C00] font-mono placeholder:text-[#A89F8B] resize-none outline-none focus:ring-0 whitespace-pre"
        ></textarea>
      </div>
      <p class="text-[10px] text-[#8B7E6A] mt-3 font-sans uppercase tracking-wider">
        Tab для выравнивания аккордов
      </p>
    </div>
  </div>
</template>
