<script setup lang="ts">
defineProps<{
  appear?: boolean;
  mode?: 'default' | 'out-in' | 'in-out';
}>();
</script>

<template>
  <Transition name="fade" :appear="appear" :mode="mode">
    <slot />
  </Transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.4s ease, transform 0.4s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
