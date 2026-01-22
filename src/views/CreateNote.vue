<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useNotesStore } from '../store/notes';
import { uploadFile } from '../api/notes';
import type { NoteType, NoteMetadata } from '../types';
import ThoughtEditor from '../components/editor/ThoughtEditor.vue';
import HarmonyEditor from '../components/editor/HarmonyEditor.vue';
import PhraseEditor from '../components/editor/PhraseEditor.vue';
import RhythmEditor from '../components/editor/RhythmEditor.vue';
import ScoreEditor from '../components/editor/ScoreEditor.vue';

const router = useRouter();
const store = useNotesStore();

const selectedType = ref<NoteType | null>(null);
const content = ref('');
const metadata = ref<NoteMetadata>({});
const saveStatus = ref<'idle' | 'saving' | 'saved'>('idle');
const autoSaveTimer = ref<number | null>(null);
const pendingFileData = ref<{ name: string; data: number[] } | null>(null);
const mouseX = ref(0);
const mouseY = ref(0);
const scrollY = ref(0);

const noteTypes: { type: NoteType; label: string; description: string; icon: string }[] = [
  { type: 'thought', label: 'Thought', description: 'Free text, idea, or reflection', icon: '✨' },
  { type: 'harmony', label: 'Harmony', description: 'Chord progression or voicings', icon: '🎵' },
  { type: 'phrase', label: 'Phrase', description: 'Audio recording with notes', icon: '📝' },
  { type: 'rhythm', label: 'Rhythm', description: 'Time signature and feel', icon: '⚡' },
  { type: 'score', label: 'Score', description: 'Image or PDF document', icon: '🎼' },
];

const headerScrolled = computed(() => scrollY.value > 50);

const cursorStyle = computed(() => ({
  left: `${mouseX.value}px`,
  top: `${mouseY.value}px`,
}));

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
  window.addEventListener('mousemove', handleMouseMove);
  window.addEventListener('scroll', handleScroll);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('scroll', handleScroll);
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }
});

function handleMouseMove(e: MouseEvent) {
  mouseX.value = e.clientX;
  mouseY.value = e.clientY;
}

function handleScroll() {
  scrollY.value = window.scrollY;
}

watch([content, metadata], () => {
  if (!selectedType.value || !content.value) return;

  saveStatus.value = 'idle';

  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }

  autoSaveTimer.value = window.setTimeout(() => {
    saveNote();
  }, 2500);
}, { deep: true });

function handleGlobalKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    if (selectedType.value && !content.value) {
      selectedType.value = null;
      return;
    }
    if (content.value) {
      saveNote().then(() => {
        goBack();
      });
    } else {
      goBack();
    }
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
  router.push({ name: 'feed' });
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
      const path = await uploadFile(
          pendingFileData.value.name,
          pendingFileData.value.data,
          fileType
      );
      metadata.value.file_path = path;
      pendingFileData.value = null;
    }

    await store.createNote({
      note_type: selectedType.value,
      content: content.value,
      metadata: metadata.value,
    });

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
</script>

<template>
  <div class="min-h-screen relative overflow-x-hidden bg-[#12141a] text-gray-200 font-sans">
    <!-- Animated Background Orbs -->
    <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden">
      <div class="absolute rounded-full blur-[100px] opacity-15 w-[600px] h-[600px] -top-[250px] -left-[200px] bg-[radial-gradient(circle,rgba(251,191,36,1)_0%,transparent_70%)] animate-[float_25s_ease-in-out_infinite]"></div>
      <div class="absolute rounded-full blur-[100px] opacity-15 w-[500px] h-[500px] top-[40%] -right-[150px] bg-[radial-gradient(circle,rgba(245,158,11,1)_0%,transparent_70%)] animate-[float_20s_ease-in-out_infinite_reverse]"></div>
      <div class="absolute rounded-full blur-[100px] opacity-15 w-[550px] h-[550px] -bottom-[200px] left-[35%] bg-[radial-gradient(circle,rgba(217,119,6,1)_0%,transparent_70%)] animate-[float_22s_ease-in-out_infinite]"></div>
    </div>

    <!-- Cursor Glow Effect -->
    <div
        class="hidden lg:block fixed w-[400px] h-[400px] rounded-full pointer-events-none -translate-x-1/2 -translate-y-1/2 z-10 transition-opacity duration-300 bg-[radial-gradient(circle,rgba(251,191,36,0.08)_0%,transparent_70%)]"
        :style="cursorStyle"
    ></div>

    <!-- Header -->
    <header
        class="fixed top-0 left-0 right-0 z-50 transition-all duration-500 border-b border-transparent"
        :class="{ 'bg-[#12141a]/85 backdrop-blur-2xl border-white/10 shadow-xl': headerScrolled }"
    >
      <div class="max-w-[1400px] mx-auto px-6 h-20 flex items-center justify-between gap-6 md:px-8">
        <button
            @click="selectedType ? (selectedType = null) : goBack()"
            class="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium text-gray-400 hover:text-white hover:bg-white/5 transition-all"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          {{ selectedType ? 'Back' : 'Cancel' }}
        </button>

        <div class="flex items-center gap-4">
          <div v-if="selectedType" class="flex items-center gap-2">
            <span v-if="saveStatus === 'saving'" class="text-xs text-gray-500">
              Saving...
            </span>
            <span v-else-if="saveStatus === 'saved'" class="flex items-center gap-1.5 text-xs text-green-400">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              Saved
            </span>
          </div>
          <button
              v-if="selectedType && content"
              @click="saveNote"
              class="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-amber-400 to-orange-600 text-[#12141a] text-sm font-bold shadow-[0_0_15px_rgba(251,191,36,0.4)] hover:shadow-[0_0_25px_rgba(251,191,36,0.6)] hover:-translate-y-0.5 active:translate-y-0 transition-all duration-300"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            Save Entry
          </button>
        </div>
      </div>
    </header>

    <main class="relative z-20 max-w-[1400px] mx-auto pt-36 px-6 pb-24 md:px-8">
      <!-- Type Selection -->
      <div v-if="!selectedType">
        <div class="text-center mb-16">
          <div class="inline-flex items-center justify-center w-20 h-20 rounded-2xl bg-gradient-to-br from-amber-400 to-orange-600 mb-6 shadow-[0_0_30px_rgba(251,191,36,0.4)]">
            <svg class="w-9 h-9 text-[#12141a]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </div>
          <h1 class="text-3xl font-bold text-white mb-4 tracking-tight">Create New Entry</h1>
          <p class="text-sm text-gray-400 max-w-md mx-auto leading-relaxed">Choose the type of entry you want to create and start capturing your musical ideas</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 max-w-5xl mx-auto">
          <button
              v-for="{ type, label, description, icon } in noteTypes"
              :key="type"
              @click="selectType(type)"
              class="group relative"
          >
            <!-- Glow Effect -->
            <div class="absolute -inset-[3px] bg-gradient-to-r from-amber-500 to-orange-600 rounded-2xl opacity-0 blur-lg transition-opacity duration-500 group-hover:opacity-50 -z-10"></div>

            <!-- Card -->
            <div class="bg-[#1a1d24] border border-white/10 rounded-2xl p-8 text-left transition-all duration-500 group-hover:border-white/20 group-hover:-translate-y-1.5 group-hover:shadow-2xl relative overflow-hidden">
              <!-- Top Accent Bar -->
              <div class="absolute top-0 left-0 right-0 h-[3px] bg-gradient-to-r from-amber-400 to-orange-600 opacity-0 transition-opacity duration-500 group-hover:opacity-100"></div>

              <div class="flex items-center justify-between mb-4">
                <div class="w-14 h-14 flex items-center justify-center text-2xl bg-[#12141a] border border-white/10 rounded-xl transition-all duration-300 group-hover:bg-gradient-to-br group-hover:from-amber-400 group-hover:to-orange-600 group-hover:border-transparent group-hover:scale-110 group-hover:rotate-[8deg] group-hover:shadow-lg">
                  {{ icon }}
                </div>
                <svg class="w-5 h-5 text-gray-600 transition-all duration-300 group-hover:text-amber-400 group-hover:translate-x-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </div>

              <h3 class="text-lg font-bold text-white mb-2">{{ label }}</h3>
              <p class="text-sm text-gray-400 leading-relaxed">{{ description }}</p>
            </div>
          </button>
        </div>

        <div class="flex items-center justify-center gap-2 mt-12">
          <span class="text-xs text-gray-500">Press</span>
          <kbd class="px-2 py-1 bg-[#1a1d24] border border-white/10 rounded text-xs text-gray-400 font-mono">Esc</kbd>
          <span class="text-xs text-gray-500">to go back</span>
        </div>
      </div>

      <!-- Editor -->
      <div v-else>
        <div class="max-w-4xl mx-auto">
          <!-- Editor Header -->
          <div class="mb-8">
            <div class="flex items-center gap-3 mb-3">
              <div class="w-12 h-12 flex items-center justify-center text-xl bg-gradient-to-br from-amber-400 to-orange-600 rounded-xl shadow-[0_0_20px_rgba(251,191,36,0.3)]">
                {{ getSelectedTypeInfo()?.icon }}
              </div>
              <div>
                <h2 class="text-xl font-bold text-white">New {{ getSelectedTypeInfo()?.label }}</h2>
                <p class="text-sm text-gray-400">{{ getSelectedTypeInfo()?.description }}</p>
              </div>
            </div>
          </div>

          <!-- Editor Card -->
          <div class="relative group">
            <!-- Glow Effect -->
            <div class="absolute -inset-[3px] bg-gradient-to-r from-amber-500 to-orange-600 rounded-2xl opacity-20 blur-lg -z-10"></div>

            <div class="bg-[#1a1d24] border border-white/10 rounded-2xl p-8 relative overflow-hidden">
              <!-- Top Accent Bar -->
              <div class="absolute top-0 left-0 right-0 h-[3px] bg-gradient-to-r from-amber-400 to-orange-600"></div>

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
          </div>

          <!-- Keyboard Shortcuts -->
          <div class="flex flex-wrap items-center justify-center gap-4 mt-8 text-xs text-gray-500">
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-1 bg-[#1a1d24] border border-white/10 rounded text-gray-400 font-mono">Cmd</kbd>
              <span>+</span>
              <kbd class="px-2 py-1 bg-[#1a1d24] border border-white/10 rounded text-gray-400 font-mono">S</kbd>
              <span>to save</span>
            </div>
            <span class="text-gray-700">·</span>
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-1 bg-[#1a1d24] border border-white/10 rounded text-gray-400 font-mono">Esc</kbd>
              <span>to go back</span>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
/* Custom Keyframe for Float */
@keyframes float {
  0% { transform: translateY(0) translateX(0); }
  50% { transform: translateY(-40px) translateX(30px); }
  100% { transform: translateY(0) translateX(0); }
}
</style>