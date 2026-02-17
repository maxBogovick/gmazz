<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import {
  uploadFileAndGetId,
  createRelease,
  listKeys,
  setApiKey,
  isTauri,
} from '../api/server';

const router = useRouter();
const isTauriEnv = isTauri();

const apiKey = ref(localStorage.getItem('gmazz_api_key') || '');
const webReleaseUser = ref(isTauriEnv ? '' : (localStorage.getItem('releaseUser') || ''));
const webReleasePass = ref('');
const webReleaseError = ref<string | null>(null);
const isAuthed = ref(false);

const isUploading = ref(false);
const newRelease = ref({
  versionName: '',
  description: '',
  file: null as File | null,
});

const saveApiKey = () => {
  setApiKey(apiKey.value);
};

const handleLogin = async () => {
  webReleaseError.value = null;
  if (!webReleaseUser.value || !webReleasePass.value) {
    webReleaseError.value = 'Введите логин и пароль';
    return;
  }
  try {
    await listKeys(webReleasePass.value);
    localStorage.setItem('adminSecret', webReleasePass.value);
    localStorage.setItem('releaseUser', webReleaseUser.value);
    webReleasePass.value = '';
    isAuthed.value = true;
  } catch (err: any) {
    webReleaseError.value = err.message || 'Неверный пароль';
  }
};

const handleLogout = () => {
  isAuthed.value = false;
  webReleasePass.value = '';
  localStorage.removeItem('adminSecret');
};

const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    newRelease.value.file = target.files[0];
  }
};

const submitRelease = async () => {
  if (!newRelease.value.file) {
    alert('Please select a .db file');
    return;
  }
  isUploading.value = true;
  try {
    const fileId = await uploadFileAndGetId(newRelease.value.file);
    await createRelease({
      file_id: fileId,
      version_name: newRelease.value.versionName || `Manual Release ${new Date().toLocaleDateString()}`,
      description: newRelease.value.description,
    });
    newRelease.value = { versionName: '', description: '', file: null };
    alert('Release uploaded');
  } catch (err: any) {
    alert('Failed: ' + err.message);
  } finally {
    isUploading.value = false;
  }
};

onMounted(() => {
  if (isTauriEnv) return;
  if (localStorage.getItem('adminSecret')) {
    isAuthed.value = true;
  }
});
</script>

<template>
  <div class="min-h-screen bg-stone-50 p-8 pt-20">
    <header class="max-w-3xl mx-auto mb-6 flex justify-between items-end gap-4">
      <div>
        <h1 class="text-3xl font-light text-stone-800 mb-2">Upload Release</h1>
        <p class="text-stone-500 text-sm">Web-only manual release upload</p>
      </div>
      <button
        @click="router.push('/admin')"
        class="px-4 py-2 bg-white border border-stone-200 rounded-lg text-stone-600 hover:bg-stone-50 transition-colors shadow-sm text-sm"
      >
        Back to Admin
      </button>
    </header>

    <main class="max-w-3xl mx-auto">
      <div v-if="isTauriEnv" class="bg-white rounded-2xl shadow-sm border border-stone-100 p-8 text-sm text-stone-600">
        Offline mode: manual upload is disabled in the desktop app. Use Sync instead.
      </div>

      <div v-else class="bg-white rounded-2xl shadow-sm border border-stone-100 p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-medium text-stone-800">Credentials</h3>
          <div class="text-xs text-stone-500">API Key + Admin password required</div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-4">
          <input
            v-model="apiKey"
            type="text"
            placeholder="API Key"
            class="px-3 py-2 border border-stone-200 rounded-lg text-sm md:col-span-2"
          />
          <button
            @click="saveApiKey"
            class="px-4 py-2 bg-stone-100 hover:bg-stone-200 rounded-lg text-sm font-medium"
          >
            Save API Key
          </button>
        </div>

        <div v-if="!isAuthed" class="grid grid-cols-1 md:grid-cols-3 gap-3">
          <input
            v-model="webReleaseUser"
            type="text"
            placeholder="Логин"
            class="px-3 py-2 border border-stone-200 rounded-lg text-sm"
          />
          <input
            v-model="webReleasePass"
            type="password"
            placeholder="Пароль"
            class="px-3 py-2 border border-stone-200 rounded-lg text-sm"
            @keyup.enter="handleLogin"
          />
          <button
            @click="handleLogin"
            class="px-4 py-2 bg-stone-800 text-white rounded-lg hover:bg-stone-900 transition-colors text-sm font-medium"
          >
            Войти
          </button>
          <div v-if="webReleaseError" class="md:col-span-3 text-red-600 text-sm">{{ webReleaseError }}</div>
        </div>

        <form v-else @submit.prevent="submitRelease" class="grid grid-cols-1 md:grid-cols-3 gap-3">
          <input
            v-model="newRelease.versionName"
            type="text"
            placeholder="Версия (например 2026.01.31)"
            class="px-3 py-2 border border-stone-200 rounded-lg text-sm md:col-span-2"
          />
          <button
            type="button"
            @click="handleLogout"
            class="px-3 py-2 bg-stone-100 hover:bg-stone-200 rounded-lg text-xs font-medium"
          >
            Выйти
          </button>
          <textarea
            v-model="newRelease.description"
            rows="2"
            placeholder="Описание релиза"
            class="px-3 py-2 border border-stone-200 rounded-lg text-sm md:col-span-3"
          />
          <input
            type="file"
            accept=".db"
            @change="handleFileSelect"
            class="px-3 py-2 border border-stone-200 rounded-lg text-sm md:col-span-2"
          />
          <button
            type="submit"
            :disabled="isUploading || !newRelease.file || !apiKey"
            class="px-4 py-2 bg-amber-600 text-white rounded-lg hover:bg-amber-700 transition-colors text-sm font-medium disabled:opacity-50"
          >
            Загрузить релиз
          </button>
        </form>
      </div>
    </main>
  </div>
</template>
