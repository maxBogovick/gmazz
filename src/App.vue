<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import "./App.css"

const router = useRouter();

onMounted(() => {
  document.addEventListener('keydown', handleGlobalHotkeys);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalHotkeys);
});

function handleGlobalHotkeys(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 'n') {
    event.preventDefault();
    if (event.shiftKey) {
      router.push({ name: 'create' });
    } else {
      router.push({ name: 'create', query: { type: 'thought' } });
    }
  }
}
</script>

<template>
  <RouterView v-slot="{ Component }">
    <Transition name="fade" mode="out-in">
      <component :is="Component" />
    </Transition>
  </RouterView>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
