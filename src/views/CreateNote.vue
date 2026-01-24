<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { uploadFile } from '../api/notes';
import type { NoteType, NoteMetadata } from '../types';
import ThoughtEditor from '../components/editor/ThoughtEditor.vue';
import HarmonyEditor from '../components/editor/HarmonyEditor.vue';
import PhraseEditor from '../components/editor/PhraseEditor.vue';
import RhythmEditor from '../components/editor/RhythmEditor.vue';
import ScoreEditor from '../components/editor/ScoreEditor.vue';

const route = useRoute();
const router = useRouter();
const store = useNotesStore();

const editingNoteId = ref<string | null>(null);
const isEditMode = computed(() => !!editingNoteId.value);
const selectedType = ref<NoteType | null>((route.query.type as NoteType) || null);
const content = ref('');
const metadata = ref<NoteMetadata>({});
const saveStatus = ref<'idle' | 'saving' | 'saved'>('idle');
const autoSaveTimer = ref<number | null>(null);
const pendingFileData = ref<{ name: string; data: number[] } | null>(null);
const mouseX = ref(0.5);
const mouseY = ref(0.5);
const time = ref(0);

const noteTypes: { type: NoteType; label: string; labelRu: string; description: string; icon: string; accent: string; gradient: string }[] = [
  {
    type: 'score',
    label: 'Original Score',
    labelRu: 'Партитура',
    description: 'Нотные записи и аранжировки',
    icon: '𝄚',
    accent: '#6B8FAD',
    gradient: 'linear-gradient(135deg, #F5F8FA 0%, #EAF1F6 100%)'
  },
  {
    type: 'harmony',
    label: 'Harmonic Study',
    labelRu: 'Гармония',
    description: 'Аккордовые последовательности',
    icon: '♮',
    accent: '#7B9E87',
    gradient: 'linear-gradient(135deg, #F5F9F6 0%, #EBF4EE 100%)'
  },
  {
    type: 'phrase',
    label: 'Recorded Phrase',
    labelRu: 'Фраза',
    description: 'Аудиозаписи мелодий',
    icon: '𝄞',
    accent: '#8B7BA8',
    gradient: 'linear-gradient(135deg, #F8F6FA 0%, #F0ECF5 100%)'
  },
  {
    type: 'rhythm',
    label: 'Rhythmic Pattern',
    labelRu: 'Ритм',
    description: 'Ритмические паттерны',
    icon: '◈',
    accent: '#B8856E',
    gradient: 'linear-gradient(135deg, #FBF6F4 0%, #F6EDE8 100%)'
  },
  {
    type: 'thought',
    label: 'Personal Note',
    labelRu: 'Заметка',
    description: 'Размышления о музыке',
    icon: '✦',
    accent: '#C4956A',
    gradient: 'linear-gradient(135deg, #FDF8F3 0%, #F9F1E8 100%)'
  },
];

let animationFrame: number;

onMounted(async () => {
  document.addEventListener('keydown', handleGlobalKeydown);
  window.addEventListener('mousemove', handleMouseMove, { passive: true });

  const animate = () => {
    time.value += 0.01;
    animationFrame = requestAnimationFrame(animate);
  };
  animate();

  // Check if editing an existing note
  const id = route.params.id as string;
  if (id) {
    editingNoteId.value = id;
    await store.fetchNote(id);
    if (store.currentNote) {
      selectedType.value = store.currentNote.note_type;
      content.value = store.currentNote.content;
      metadata.value = store.currentNote.metadata || {};
    }
  }
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
  window.removeEventListener('mousemove', handleMouseMove);
  if (animationFrame) cancelAnimationFrame(animationFrame);
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }
});

function handleMouseMove(e: MouseEvent) {
  requestAnimationFrame(() => {
    mouseX.value = e.clientX / window.innerWidth;
    mouseY.value = e.clientY / window.innerHeight;
  });
}

watch(() => route.query.type, (newType) => {
  if (newType) {
    selectedType.value = newType as NoteType;
  }
});

watch([content, metadata], () => {
  if (!selectedType.value || !content.value) return;
  saveStatus.value = 'idle';
  // Auto-save removed as per request
}, { deep: true });

function handleGlobalKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    if (saveStatus.value === 'idle' && content.value) {
       // Optional: confirm discard? 
       // For now, just go back as per "no auto save". 
       // User can stay and save if they want.
       if(!confirm('Есть несохраненные изменения. Выйти без сохранения?')) return;
    }
    goBack();
  }

  if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault();
    saveNote();
  }
}

function selectType(type: NoteType) {
  selectedType.value = type;
}

function goBack() {
  if (editingNoteId.value) {
    router.push({ name: 'note', params: { id: editingNoteId.value } });
  } else {
    router.push({ name: 'feed' });
  }
}

async function handleFilePath(fileName: string, fileData: number[]) {
  pendingFileData.value = { name: fileName, data: fileData };
}

async function saveNote() {
  if (!selectedType.value || !content.value) return;

  saveStatus.value = 'saving';

  try {
    if (pendingFileData.value) {
      const fileType = selectedType.value === 'phrase' ? 'audio' : 'scores';
      // Convert number[] back to Uint8Array then Blob then File
      const uint8Array = new Uint8Array(pendingFileData.value.data);
      const blob = new Blob([uint8Array]); 
      const file = new File([blob], pendingFileData.value.name, { 
          type: fileType === 'audio' ? 'audio/wav' : 'application/pdf' // inferred simplistic type
      });
      
      const path = await uploadFile(file);
      metadata.value.file_path = path;
      pendingFileData.value = null;
    }

    if (editingNoteId.value) {
      await store.updateNote(editingNoteId.value, {
        content: content.value,
        metadata: metadata.value,
      });
    } else {
      await store.createNote({
        note_type: selectedType.value,
        content: content.value,
        metadata: metadata.value,
      });
    }

    saveStatus.value = 'saved';

    setTimeout(() => {
      goBack();
    }, 300);
  } catch (error) {
    console.error('Failed to save note:', error);
    saveStatus.value = 'idle';
  }
}

function handleEditorSave() {
  saveNote();
}

function updateTimeSignature(value: string) {
  metadata.value = { ...metadata.value, time_signature: value };
}

function getSelectedTypeInfo() {
  return noteTypes.find(t => t.type === selectedType.value);
}

const ambientStyle = computed(() => {
  const selectedInfo = getSelectedTypeInfo();
  if (!selectedInfo) {
    return {
      background: `
        radial-gradient(ellipse 100% 70% at ${30 + mouseX.value * 25}% ${20 + mouseY.value * 25}%, rgba(220, 200, 180, 0.15) 0%, transparent 65%),
        radial-gradient(ellipse 80% 60% at ${70 - mouseX.value * 20}% ${60 + mouseY.value * 20}%, rgba(200, 180, 160, 0.1) 0%, transparent 65%),
        linear-gradient(180deg, #FAF7F2 0%, #F6F2EC 50%, #F2EDE6 100%)
      `
    };
  }

  const r = parseInt(selectedInfo.accent.slice(1, 3), 16);
  const g = parseInt(selectedInfo.accent.slice(3, 5), 16);
  const b = parseInt(selectedInfo.accent.slice(5, 7), 16);

  return {
    background: `
      radial-gradient(ellipse 100% 70% at ${30 + mouseX.value * 25}% ${20 + mouseY.value * 25}%, rgba(${r}, ${g}, ${b}, 0.12) 0%, transparent 65%),
      radial-gradient(ellipse 80% 60% at ${70 - mouseX.value * 20}% ${60 + mouseY.value * 20}%, rgba(${r}, ${g}, ${b}, 0.08) 0%, transparent 65%),
      linear-gradient(180deg, #FAF7F2 0%, #F6F2EC 50%, #F2EDE6 100%)
    `
  };
});
</script>

<template>
  <div class="min-h-screen overflow-x-hidden" :style="ambientStyle">

    <!-- Enhanced Atmospheric Layers -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      <div
          class="absolute -top-1/3 -left-1/4 w-[1000px] h-[1000px] rounded-full opacity-30 will-change-transform blur-[100px]"
          :style="{
          background: selectedType
            ? `radial-gradient(circle, ${getSelectedTypeInfo()?.accent}30 0%, ${getSelectedTypeInfo()?.accent}15 40%, transparent 70%)`
            : 'radial-gradient(circle, rgba(220, 200, 180, 0.4) 0%, rgba(220, 200, 180, 0.2) 40%, transparent 70%)',
          transform: `translate3d(${mouseX * 40}px, ${mouseY * 40}px, 0) scale(${1 + Math.sin(time) * 0.08})`
        }"
      />

      <div
          class="absolute -bottom-1/3 -right-1/4 w-[800px] h-[800px] rounded-full opacity-25 will-change-transform blur-[100px]"
          :style="{
          background: selectedType
            ? `radial-gradient(circle, ${getSelectedTypeInfo()?.accent}25 0%, ${getSelectedTypeInfo()?.accent}10 40%, transparent 70%)`
            : 'radial-gradient(circle, rgba(200, 180, 160, 0.3) 0%, rgba(200, 180, 160, 0.15) 40%, transparent 70%)',
          transform: `translate3d(${-mouseX * 30}px, ${-mouseY * 30}px, 0) scale(${1 + Math.cos(time * 0.7) * 0.08})`
        }"
      />
    </div>

    <!-- Floating Musical Notes -->
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      <div
          v-for="i in 25"
          :key="i"
          class="absolute will-change-transform font-serif opacity-20"
          :class="i % 3 === 0 ? 'text-2xl' : 'text-xl'"
          :style="{
          left: `${(i * 7) % 100}%`,
          top: `${(i * 9 + 15) % 100}%`,
          color: selectedType ? getSelectedTypeInfo()?.accent : '#D4A574',
          animation: `float-musical-${i % 3} ${8 + i % 5}s ease-in-out infinite`,
          animationDelay: `${i * 0.2}s`,
        }"
      >
        {{ ['𝅝', '𝅗𝅥', '♩', '♪', '♫', '𝄞'][i % 6] }}
      </div>
    </div>

    <!-- Header with Enhanced Glassmorphism -->
    <header
        data-tauri-drag-region
        class="fixed top-0 left-0 right-0 z-50 backdrop-blur-2xl border-b transition-all duration-700"
        :style="{
        backgroundColor: 'rgba(250, 247, 242, 0.9)',
        borderColor: selectedType ? `${getSelectedTypeInfo()?.accent}30` : 'rgba(200, 180, 160, 0.3)',
        boxShadow: '0 8px 32px rgba(0,0,0,0.06)'
      }"
    >
      <div class="max-w-6xl mx-auto px-8 lg:px-16 h-20 flex items-center justify-between pointer-events-none">
        <button
            @click="isEditMode ? goBack() : (selectedType ? (selectedType = null) : goBack())"
            class="flex items-center gap-3 text-sm text-stone-600 hover:text-amber-700 transition-all duration-300 pointer-events-auto group"
        >
          <span class="text-xl group-hover:-translate-x-1 transition-transform duration-300">←</span>
          <span class="font-medium tracking-tight">{{ isEditMode ? 'Назад к записи' : (selectedType ? 'К выбору категории' : 'Вернуться в Архив') }}</span>
        </button>

        <div class="flex items-center gap-5 pointer-events-auto">
          <!-- Save Status -->
          <div v-if="selectedType" class="flex items-center gap-3">
            <div
                class="w-3 h-3 rounded-full transition-all duration-500"
                :class="{
                'animate-pulse': saveStatus === 'saving',
                'shadow-[0_0_12px_rgba(0,0,0,0.3)]': saveStatus === 'saved',
              }"
                :style="{
                backgroundColor: saveStatus === 'saving' ? '#D4CAB5' :
                                saveStatus === 'saved' ? getSelectedTypeInfo()?.accent :
                                '#E0D9C8'
              }"
            ></div>
            <span v-if="saveStatus === 'saving'" class="text-[10px] text-stone-600 uppercase tracking-[0.25em] font-sans font-bold">
              Сохранение...
            </span>
            <span v-else-if="saveStatus === 'saved'" class="text-[10px] uppercase tracking-[0.25em] font-sans font-bold" :style="{ color: getSelectedTypeInfo()?.accent }">
              Сохранено
            </span>
          </div>

          <!-- Manual Save Button -->
          <button
              v-if="selectedType && content"
              @click="saveNote"
              class="px-6 py-2.5 text-white hover:scale-105 text-xs uppercase tracking-[0.25em] font-sans font-bold transition-all duration-300 rounded-xl shadow-lg hover:shadow-xl"
              :style="{
              background: `linear-gradient(135deg, ${getSelectedTypeInfo()?.accent} 0%, ${getSelectedTypeInfo()?.accent}CC 100%)`,
              boxShadow: `0 4px 12px ${getSelectedTypeInfo()?.accent}40`
            }"
              title="Сохранить (Cmd+S)"
          >
            Сохранить
          </button>
        </div>
      </div>
    </header>

    <main class="relative z-10 max-w-6xl mx-auto pt-36 px-8 lg:px-16 pb-32">

      <!-- Type Selection -->
      <div v-if="!selectedType" class="fade-in">
        <!-- Enhanced Section Header -->
        <div class="text-center mb-20">
          <div class="flex items-center justify-center gap-5 mb-10">
            <div class="h-[2px] w-16 bg-gradient-to-r from-transparent via-amber-300 to-amber-300 rounded-full"></div>
            <span class="text-xs uppercase tracking-[0.35em] text-amber-600 font-sans font-bold">Новая Запись</span>
            <div class="h-[2px] w-16 bg-gradient-to-l from-transparent via-amber-300 to-amber-300 rounded-full"></div>
          </div>

          <h1 class="text-5xl lg:text-6xl font-light mb-6 text-stone-800 tracking-tight">
            Добавить в <span class="text-amber-700">Архив</span>
          </h1>
          <p class="text-stone-500 font-light text-xl max-w-2xl mx-auto leading-relaxed">
            Выберите категорию для новой музыкальной записи
          </p>
        </div>

        <!-- Enhanced Type Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-7 max-w-5xl mx-auto">
          <button
              v-for="{ type, label, labelRu, description, icon, accent, gradient } in noteTypes"
              :key="type"
              @click="selectType(type)"
              class="group relative perspective-1000"
          >
            <div
                class="relative rounded-[28px] p-2 transition-all duration-700 overflow-hidden preserve-3d shadow-lg hover:shadow-2xl"
                :style="{
                background: 'rgba(255, 255, 255, 0.9)',
                transform: 'translateZ(0)'
              }"
            >
              <!-- Animated glow on hover -->
              <div
                  class="absolute inset-0 rounded-[28px] opacity-0 group-hover:opacity-100 transition-opacity duration-700"
                  :style="{
                  background: `radial-gradient(circle at ${mouseX * 100}% ${mouseY * 100}%, ${accent}25 0%, ${accent}10 40%, transparent 70%)`
                }"
              ></div>

              <div class="relative rounded-[24px] min-h-[240px] flex flex-col items-center justify-center text-center p-8 overflow-hidden" :style="{ background: gradient }">

                <!-- Enhanced Corner Ornaments -->
                <div class="absolute top-4 left-4 w-6 h-6 border-t-2 border-l-2 rounded-tl-lg transition-all duration-500" :style="{ borderColor: accent + '50' }"></div>
                <div class="absolute top-4 right-4 w-6 h-6 border-t-2 border-r-2 rounded-tr-lg transition-all duration-500" :style="{ borderColor: accent + '50' }"></div>
                <div class="absolute bottom-4 left-4 w-6 h-6 border-b-2 border-l-2 rounded-bl-lg transition-all duration-500" :style="{ borderColor: accent + '50' }"></div>
                <div class="absolute bottom-4 right-4 w-6 h-6 border-b-2 border-r-2 rounded-br-lg transition-all duration-500" :style="{ borderColor: accent + '50' }"></div>

                <!-- Large Icon with 3D effect -->
                <div
                    class="text-7xl mb-7 font-serif group-hover:scale-125 group-hover:rotate-12 transition-all duration-700 drop-shadow-lg relative z-10"
                    :style="{
                    color: accent + '70',
                    transform: `translateZ(20px)`
                  }"
                >
                  {{ icon }}
                </div>

                <!-- Labels with enhanced typography -->
                <h3 class="text-2xl font-light text-stone-800 mb-2 group-hover:scale-105 transition-all duration-500 relative z-10" :style="{ color: accent }">
                  {{ labelRu }}
                </h3>
                <div class="text-[10px] uppercase tracking-[0.25em] text-stone-500 mb-4 font-sans font-bold">
                  {{ label }}
                </div>
                <p class="text-sm text-stone-600 group-hover:text-stone-700 transition-colors leading-relaxed">
                  {{ description }}
                </p>

                <!-- Hover indicator -->
                <div class="absolute bottom-6 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-all duration-500 group-hover:translate-y-0 translate-y-2">
                  <span class="text-xs font-bold uppercase tracking-wider flex items-center gap-2" :style="{ color: accent }">
                    Выбрать <span class="text-base">→</span>
                  </span>
                </div>
              </div>
            </div>

            <!-- Outer glow on hover -->
            <div class="absolute -inset-3 rounded-[32px] opacity-0 group-hover:opacity-100 transition-opacity duration-1000 pointer-events-none blur-xl" :style="{ background: `radial-gradient(circle, ${accent}15 0%, transparent 70%)` }"></div>
          </button>
        </div>

        <!-- Enhanced Keyboard Hint -->
        <div class="text-center mt-16">
          <div class="inline-flex items-center gap-3 text-xs text-stone-500 font-sans">
            <kbd class="px-4 py-2 bg-white/70 border-2 border-stone-300 rounded-xl text-stone-600 tracking-wider font-semibold shadow-sm">ESC</kbd>
            <span>для возврата в архив</span>
          </div>
        </div>
      </div>

      <!-- Editor -->
      <div v-else class="fade-in">
        <!-- Enhanced Editor Header -->
        <div class="mb-14">
          <div class="flex items-center gap-5 mb-8">
            <div class="h-[2px] flex-1 bg-gradient-to-r from-transparent to-stone-300 rounded-full"></div>
            <div class="flex items-center gap-5 text-xs uppercase tracking-[0.35em] font-sans font-bold" :style="{ color: getSelectedTypeInfo()?.accent }">
              <span>{{ getSelectedTypeInfo()?.label }}</span>
            </div>
            <div class="h-[2px] flex-1 bg-gradient-to-l from-transparent to-stone-300 rounded-full"></div>
          </div>

          <div class="text-center">
            <div
                class="text-8xl mb-6 inline-block animate-float font-serif drop-shadow-lg"
                :style="{
                color: getSelectedTypeInfo()?.accent + '80',
                transform: `scale(${1 + Math.sin(time) * 0.05})`
              }"
            >
              {{ getSelectedTypeInfo()?.icon }}
            </div>
            <h2 class="text-4xl lg:text-5xl font-light text-stone-800 tracking-tight">
              {{ isEditMode ? 'Редактирование' : 'Новая' }} {{ getSelectedTypeInfo()?.labelRu }}
            </h2>
          </div>
        </div>

        <!-- Enhanced Editor Frame -->
        <div
            class="relative rounded-[32px] p-2 shadow-2xl max-w-4xl mx-auto overflow-hidden group"
            :style="{
            background: 'rgba(255, 255, 255, 0.95)',
            boxShadow: `0 20px 60px ${getSelectedTypeInfo()?.accent}15, 0 8px 20px rgba(0,0,0,0.08)`
          }"
        >
          <!-- Animated border glow -->
          <div
              class="absolute inset-0 rounded-[32px] opacity-40 group-hover:opacity-60 transition-opacity duration-700"
              :style="{
              background: `linear-gradient(135deg, ${getSelectedTypeInfo()?.accent}15 0%, transparent 100%)`
            }"
          ></div>

          <div class="relative rounded-[28px] overflow-hidden" :style="{ background: getSelectedTypeInfo()?.gradient }">
            <!-- Enhanced Corner Details -->
            <div class="absolute top-6 left-6 w-8 h-8 border-t-2 border-l-2 rounded-tl-lg transition-all duration-500 z-10" :style="{ borderColor: getSelectedTypeInfo()?.accent + '60' }"></div>
            <div class="absolute top-6 right-6 w-8 h-8 border-t-2 border-r-2 rounded-tr-lg transition-all duration-500 z-10" :style="{ borderColor: getSelectedTypeInfo()?.accent + '60' }"></div>
            <div class="absolute bottom-6 left-6 w-8 h-8 border-b-2 border-l-2 rounded-bl-lg transition-all duration-500 z-10" :style="{ borderColor: getSelectedTypeInfo()?.accent + '60' }"></div>
            <div class="absolute bottom-6 right-6 w-8 h-8 border-b-2 border-r-2 rounded-br-lg transition-all duration-500 z-10" :style="{ borderColor: getSelectedTypeInfo()?.accent + '60' }"></div>

            <div class="p-10 lg:p-14 relative z-20">
              <component
                  :is="selectedType === 'thought' ? ThoughtEditor :
                     selectedType === 'harmony' ? HarmonyEditor :
                     selectedType === 'phrase' ? PhraseEditor :
                     selectedType === 'rhythm' ? RhythmEditor :
                     ScoreEditor"
                  v-model="content"
                  :time-signature="metadata.time_signature"
                  :file-path="metadata.file_path"
                  @save="handleEditorSave"
                  @update:time-signature="updateTimeSignature"
                  @update:file-path="handleFilePath"
              />
            </div>

            <!-- Enhanced Footer -->
            <div class="px-10 lg:px-14 pb-8 relative z-20">
              <div class="pt-8 border-t-2 flex justify-between items-center text-[10px] uppercase tracking-[0.3em] font-sans transition-colors duration-500" :style="{ borderColor: getSelectedTypeInfo()?.accent + '30' }">
                <span class="text-stone-600 font-bold">Архив Gmazz</span>
                <div class="flex items-center gap-6">
                  <span class="hidden sm:inline text-stone-500">Cmd+S для сохранения</span>
                  <span class="font-bold" :style="{ color: getSelectedTypeInfo()?.accent }">{{ new Date().toLocaleDateString('ru-RU') }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Keyboard Shortcuts -->
        <div class="text-center mt-12">
          <div class="inline-flex items-center gap-6 text-xs text-stone-500 font-sans">
            <div class="flex items-center gap-2">
              <kbd class="px-3 py-1.5 bg-white/70 border border-stone-300 rounded-lg text-stone-600 tracking-wider font-semibold shadow-sm">Cmd</kbd>
              <span>+</span>
              <kbd class="px-3 py-1.5 bg-white/70 border border-stone-300 rounded-lg text-stone-600 tracking-wider font-semibold shadow-sm">S</kbd>
              <span>сохранить</span>
            </div>
            <span class="text-stone-300">•</span>
            <div class="flex items-center gap-2">
              <kbd class="px-3 py-1.5 bg-white/70 border border-stone-300 rounded-lg text-stone-600 tracking-wider font-semibold shadow-sm">ESC</kbd>
              <span>выход</span>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
@keyframes float-musical-0 {
  0%, 100% {
    transform: translate3d(0, 0, 0) rotate(0deg);
    opacity: 0.15;
  }
  25% {
    transform: translate3d(20px, -30px, 0) rotate(90deg);
    opacity: 0.4;
  }
  50% {
    transform: translate3d(-10px, -20px, 0) rotate(180deg);
    opacity: 0.2;
  }
  75% {
    transform: translate3d(10px, -40px, 0) rotate(270deg);
    opacity: 0.35;
  }
}

@keyframes float-musical-1 {
  0%, 100% {
    transform: translate3d(0, 0, 0) rotate(0deg);
    opacity: 0.2;
  }
  33% {
    transform: translate3d(-15px, -25px, 0) rotate(120deg);
    opacity: 0.45;
  }
  66% {
    transform: translate3d(15px, -35px, 0) rotate(240deg);
    opacity: 0.25;
  }
}

@keyframes float-musical-2 {
  0%, 100% {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.18;
  }
  50% {
    transform: translate3d(12px, -45px, 0) scale(1.3);
    opacity: 0.4;
  }
}

@keyframes float {
  0%, 100% {
    transform: translateY(0px) scale(1);
  }
  50% {
    transform: translateY(-15px) scale(1.05);
  }
}

.fade-in {
  animation: fadeIn 0.8s ease-out forwards;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-float {
  animation: float 4s ease-in-out infinite;
}

.perspective-1000 {
  perspective: 1000px;
}

.preserve-3d {
  transform-style: preserve-3d;
}

* {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.will-change-transform {
  will-change: transform;
}

/* Smooth transitions for all interactive elements */
button {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

button:active {
  transform: scale(0.98);
}

/* Smooth scroll behavior */
html {
  scroll-behavior: smooth;
}

/* Enhanced focus states for accessibility */
button:focus-visible,
kbd:focus-visible {
  outline: 2px solid rgba(196, 149, 106, 0.6);
  outline-offset: 3px;
  border-radius: 8px;
}

/* Prevent text selection on interactive elements */
button,
kbd {
  user-select: none;
  -webkit-user-select: none;
}

/* Optimize rendering performance */
.group > div {
  backface-visibility: hidden;
  transform: translateZ(0);
}

/* Custom scrollbar for the page */
::-webkit-scrollbar {
  width: 12px;
}

::-webkit-scrollbar-track {
  background: rgba(245, 240, 235, 0.5);
  border-radius: 6px;
}

::-webkit-scrollbar-thumb {
  background: rgba(196, 149, 106, 0.4);
  border-radius: 6px;
  transition: background 0.3s ease;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(196, 149, 106, 0.6);
}

/* Smooth animation delays for staggered effects */
.grid > button:nth-child(1) {
  animation-delay: 0s;
}

.grid > button:nth-child(2) {
  animation-delay: 0.1s;
}

.grid > button:nth-child(3) {
  animation-delay: 0.2s;
}

.grid > button:nth-child(4) {
  animation-delay: 0.3s;
}

.grid > button:nth-child(5) {
  animation-delay: 0.4s;
}

/* Enhance kbd elements */
kbd {
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
}

/* Loading state animation */
@keyframes pulse-slow {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.animate-pulse {
  animation: pulse-slow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

/* Hover lift effect for cards */
@keyframes lift {
  0% {
    transform: translateY(0) scale(1);
  }
  100% {
    transform: translateY(-8px) scale(1.02);
  }
}

.group:hover {
  animation: lift 0.3s ease-out forwards;
}

/* Subtle shadow animation */
@keyframes shadow-pulse {
  0%, 100% {
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.08);
  }
  50% {
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12);
  }
}

/* Text gradient animation for special elements */
@keyframes gradient-shift {
  0%, 100% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
}

/* Ripple effect on click */
@keyframes ripple {
  0% {
    transform: scale(0);
    opacity: 1;
  }
  100% {
    transform: scale(4);
    opacity: 0;
  }
}

/* Ensure smooth rendering on mobile */
@media (max-width: 768px) {
  .grid {
    gap: 1.5rem;
  }

  button {
    -webkit-tap-highlight-color: transparent;
  }
}

/* Improve text rendering */
body {
  text-rendering: optimizeLegibility;
  font-feature-settings: "kern" 1;
  font-kerning: normal;
}

/* Gradient text for special headings */
.gradient-text {
  background: linear-gradient(135deg, #C4956A 0%, #B8860B 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* Ensure proper z-index stacking */
.relative {
  isolation: isolate;
}

/* Add subtle backdrop filter support check */
@supports (backdrop-filter: blur(20px)) or (-webkit-backdrop-filter: blur(20px)) {
  header {
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
}

/* Fallback for browsers without backdrop-filter */
@supports not ((backdrop-filter: blur(20px)) or (-webkit-backdrop-filter: blur(20px))) {
  header {
    background-color: rgba(250, 247, 242, 0.95) !important;
  }
}
</style>