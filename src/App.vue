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
  const isMod = event.metaKey || event.ctrlKey;

  // Cmd + N -> New Thought (direct)
  if (isMod && event.key === 'n' && !event.shiftKey) {
    event.preventDefault();
    router.push({ name: 'create', query: { type: 'thought' } });
  }

  // Cmd + Shift + N -> Select Type
  if (isMod && event.key === 'n' && event.shiftKey) {
    event.preventDefault();
    router.push({ name: 'create' });
  }

  // Note: Escape is handled by individual views (CreateNote, SingleNote)
  // for more nuanced behavior (save before exit, etc.)

  // Cmd + S -> Global Save Trigger
  if (isMod && event.key === 's') {
    event.preventDefault();
    // Dispatch a custom event that editors can listen to
    window.dispatchEvent(new CustomEvent('jazz-save'));
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
