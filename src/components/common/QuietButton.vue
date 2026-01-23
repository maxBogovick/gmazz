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
    class="px-4 py-2 text-sm text-[#2C2416] hover:text-[#A67C00] transition-colors duration-300 disabled:opacity-40 disabled:cursor-not-allowed"
  >
    <slot />
  </button>
</template>
