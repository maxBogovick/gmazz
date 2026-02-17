<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import "./App.css"

const router = useRouter();
const isTauri = !!(window as any).__TAURI_INTERNALS__;

onMounted(() => {
  document.addEventListener('keydown', handleGlobalHotkeys);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalHotkeys);
});

function handleGlobalHotkeys(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null;
  if (target && (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))) {
    return;
  }
  const isMod = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();

  // Cmd + N -> New Thought (direct)
  if (isMod && key === 'n' && !event.shiftKey) {
    event.preventDefault();
    router.push({ name: 'create', query: { type: 'thought' } });
  }

  // Cmd + Shift + N -> Select Type
  if (isMod && key === 'n' && event.shiftKey) {
    event.preventDefault();
    router.push({ name: 'create' });
  }

  // Note: Escape is handled by individual views (CreateNote, SingleNote)
  // for more nuanced behavior (save before exit, etc.)

  // Cmd + S -> Global Save Trigger
  if (isMod && key === 's') {
    event.preventDefault();
    // Dispatch a custom event that editors can listen to
    window.dispatchEvent(new CustomEvent('jazz-save'));
  }
}
</script>

<template>
  <div>
    <RouterView v-slot="{ Component }">
      <Transition name="fade" mode="out-in">
        <component :is="Component" />
      </Transition>
    </RouterView>

    <!-- Admin Button -->
    <button 
      v-if="isTauri"
      @click="router.push({ name: 'admin' })"
      class="fixed bottom-6 right-6 z-[100] w-12 h-12 bg-stone-800 text-white rounded-full flex items-center justify-center shadow-xl hover:bg-stone-900 hover:scale-110 transition-all duration-300 group"
      title="Admin Panel"
    >
      <span class="text-xl group-hover:rotate-12 transition-transform">⚙️</span>
      <div class="absolute right-full mr-4 px-3 py-1.5 bg-stone-800 text-[11px] uppercase tracking-widest text-white rounded-lg opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
        Панель управления
      </div>
    </button>
  </div>
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
