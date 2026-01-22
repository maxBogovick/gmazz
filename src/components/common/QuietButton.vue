<script setup lang="ts">
defineProps<{
  type?: 'button' | 'submit';
  disabled?: boolean;
}>();
</script>

<template>
  <button
    :type="type || 'button'"
    :disabled="disabled"
    class="px-4 py-2 text-sm text-[--color-text-secondary] hover:text-[--color-accent-brass] transition-colors duration-300 disabled:opacity-40 disabled:cursor-not-allowed"
  >
    <slot />
  </button>
</template>
