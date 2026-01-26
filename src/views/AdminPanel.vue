<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { 
  getReleases, getLatestRelease, createRelease, uploadFileAndGetId, 
  listKeys, createKey, updateKeyStatus,
  type Release, type AppKey, type CreatedKey
} from '../api/server';
import { syncDatabase } from '../api/notes';
import { useRouter } from 'vue-router';

const router = useRouter();
const isTauri = !!(window as any).__TAURI_INTERNALS__;

// Auth
const adminSecret = ref(localStorage.getItem('adminSecret') || '');
const apiKey = ref(localStorage.getItem('gmazz_api_key') || import.meta.env.VITE_API_KEY || '');
const isAuthenticated = ref(false);

const saveApiKey = () => {
  localStorage.setItem('gmazz_api_key', apiKey.value);
  // Reload page to apply to server.ts constant (which evaluates on module load)
  // Or better, just alert user they need to reload. 
  // server.ts constant is evaluated once. 
  window.location.reload();
};

// Tabs
type Tab = 'releases' | 'keys';
const currentTab = ref<Tab>('releases');

// Data
const releases = ref<Release[]>([]);
const keys = ref<AppKey[]>([]);
const latestRelease = ref<Release | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

// Releases Modal
const showReleaseModal = ref(false);
const isUploading = ref(false);
const newRelease = ref({
  versionName: '',
  description: '',
  file: null as File | null
});
const fileInput = ref<HTMLInputElement | null>(null);

// Keys Modal
const showKeyModal = ref(false);
const newKeyName = ref('');
const createdKeyResult = ref<CreatedKey | null>(null);

const formatDate = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleString();
};

const formatSize = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const getDownloadUrl = (fileId: string) => {
  return `${import.meta.env.VITE_SERVER_URL || 'http://localhost:8080'}/v1/files/${fileId}`;
};

const checkAuth = async () => {
  if (!adminSecret.value) {
    isAuthenticated.value = false;
    return;
  }
  // Try to load data to verify secret
  await loadData();
};

const loadData = async () => {
  loading.value = true;
  error.value = null;
  try {
    if (currentTab.value === 'releases') {
      const [releasesData, latestData] = await Promise.all([
        getReleases(50, 0),
        getLatestRelease()
      ]);
      releases.value = releasesData;
      latestRelease.value = latestData;
    } else {
      // Keys require admin secret
      keys.value = await listKeys(adminSecret.value);
    }
    isAuthenticated.value = true;
    localStorage.setItem('adminSecret', adminSecret.value);
  } catch (err: any) {
    if (err.message.includes('401') || err.message.includes('Unauthorized')) {
      isAuthenticated.value = false;
      if (currentTab.value === 'keys') error.value = 'Invalid Admin Secret';
    } else {
      error.value = err.message || 'Failed to load data';
    }
  } finally {
    loading.value = false;
  }
};

watch(currentTab, () => {
  if (currentTab.value === 'keys' && !isAuthenticated.value && !adminSecret.value) {
    // Prompt login
    return;
  }
  loadData();
});

onMounted(() => {
  // Releases are public-ish (via API key), Keys are Admin-only
  // We can load releases without admin secret potentially if API KEY is set in env
  // But for this unified panel, let's just load releases first.
  loadData();
});

const isCurrent = (id: string) => latestRelease.value?.id === id;

// --- Releases Logic ---
const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    newRelease.value.file = target.files[0];
  }
};

const openReleaseModal = () => {
  newRelease.value = { versionName: '', description: '', file: null };
  if (fileInput.value) fileInput.value.value = '';
  showReleaseModal.value = true;
};

const closeReleaseModal = () => {
  showReleaseModal.value = false;
};

const submitRelease = async () => {
  console.log('submitRelease called');
  if (!newRelease.value.file) {
    alert('Please select a .db file');
    return;
  }

  isUploading.value = true;
  try {
    console.log('Uploading file...');
    // 1. Upload File
    const fileId = await uploadFileAndGetId(newRelease.value.file);
    console.log('File uploaded, ID:', fileId);

    // 2. Create Release
    console.log('Creating release...');
    await createRelease({
      file_id: fileId,
      version_name: newRelease.value.versionName || `Manual Release ${new Date().toLocaleDateString()}`,
      description: newRelease.value.description
    });
    console.log('Release created');

    closeReleaseModal();
    await loadData();
  } catch (err: any) {
    console.error('submitRelease error:', err);
    alert('Failed: ' + err.message);
  } finally {
    isUploading.value = false;
  }
};

const handleRollback = async (release: Release) => {
  if (!confirm(`Rollback to "${release.version_name || 'Untitled'}"?`)) return;
  try {
    await createRelease({
      file_id: release.file_id,
      version_name: `Rollback to ${release.version_name || 'Previous'}`,
      description: `Rolled back from release ${latestRelease.value?.version_name || 'current'}`
    });
    await loadData();
  } catch (err: any) {
    alert('Rollback failed: ' + err.message);
  }
};

// --- Keys Logic ---
const openKeyModal = () => {
  newKeyName.value = '';
  createdKeyResult.value = null;
  showKeyModal.value = true;
};

const submitKey = async () => {
  if (!newKeyName.value) return;
  try {
    createdKeyResult.value = await createKey(adminSecret.value, newKeyName.value);
    await loadData();
  } catch (err: any) {
    alert('Failed to create key: ' + err.message);
  }
};

const toggleKeyStatus = async (key: AppKey) => {
  const action = key.is_active ? 'Revoke' : 'Activate';
  if (!confirm(`${action} key "${key.name}"?`)) return;
  
  try {
    await updateKeyStatus(adminSecret.value, key.id, !key.is_active);
    await loadData();
  } catch (err: any) {
    alert(`Failed to ${action} key: ${err.message}`);
  }
};

const handleLogin = () => {
  if (adminSecret.value) {
    checkAuth();
  }
};

const handleSync = async () => {
  //if (!confirm('This will bundle your local database and files and upload them as a new Release. Continue?')) return;
  loading.value = true;
  try {
    const id = await syncDatabase();
    await loadData();
    alert('Sync successful! Release created: ' + id);
  } catch (e: any) {
    alert('Sync failed: ' + e);
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="min-h-screen bg-stone-50 p-8 pt-20 relative">
    <header class="max-w-6xl mx-auto mb-8 flex flex-col md:flex-row justify-between items-end gap-4">
      <div>
        <h1 class="text-3xl font-light text-stone-800 mb-2">Admin Panel</h1>
        <div class="flex gap-6 text-sm">
          <button 
            @click="currentTab = 'releases'"
            class="pb-2 border-b-2 transition-colors"
            :class="currentTab === 'releases' ? 'border-amber-600 text-amber-700 font-medium' : 'border-transparent text-stone-500 hover:text-stone-800'"
          >
            Releases
          </button>
          <button 
            @click="currentTab = 'keys'"
            class="pb-2 border-b-2 transition-colors"
            :class="currentTab === 'keys' ? 'border-amber-600 text-amber-700 font-medium' : 'border-transparent text-stone-500 hover:text-stone-800'"
          >
            API Keys
          </button>
        </div>
      </div>
      
      <div class="flex flex-col items-end gap-2">
        <div class="flex gap-2">
            <input 
              v-model="apiKey"
              type="text" 
              placeholder="Session API Key"
              class="px-3 py-2 border border-stone-200 rounded-lg text-xs w-64"
            />
            <button @click="saveApiKey" class="px-3 py-2 bg-stone-100 hover:bg-stone-200 rounded-lg text-xs font-medium">Save & Reload</button>
        </div>
        <button 
          @click="router.push('/')"
          class="px-4 py-2 bg-white border border-stone-200 rounded-lg text-stone-600 hover:bg-stone-50 transition-colors shadow-sm text-sm"
        >
          Back to App
        </button>
      </div>
    </header>

    <!-- Content -->
    <main class="max-w-6xl mx-auto">
      
      <!-- Login for Keys -->
      <div v-if="currentTab === 'keys' && !isAuthenticated" class="max-w-md mx-auto mt-20 bg-white p-8 rounded-2xl shadow-lg text-center">
        <h2 class="text-xl font-medium text-stone-800 mb-4">Admin Access Required</h2>
        <p class="text-stone-500 mb-6 text-sm">Please enter the Admin Secret to manage API keys.</p>
        <input 
          v-model="adminSecret"
          type="password" 
          placeholder="Admin Secret"
          class="w-full px-4 py-3 border border-stone-200 rounded-lg mb-4 focus:ring-2 focus:ring-amber-500/20 outline-none"
          @keyup.enter="handleLogin"
        />
        <button 
          @click="handleLogin"
          class="w-full py-3 bg-stone-800 text-white rounded-lg hover:bg-stone-900 transition-colors font-medium"
        >
          Unlock
        </button>
        <p v-if="error" class="mt-4 text-red-600 text-sm">{{ error }}</p>
      </div>

      <!-- Main Interface -->
      <div v-else>
        
        <!-- Actions Bar -->
        <div class="flex justify-between items-center mb-6">
          <p class="text-stone-500 text-sm">
            {{ currentTab === 'releases' ? 'Manage database versions' : 'Manage client access keys' }}
          </p>
          <div class="flex gap-3">
            <button 
              v-if="currentTab === 'releases' && isTauri"
              @click="handleSync"
              class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors shadow-sm font-medium"
            >
              Sync (Create Release)
            </button>
            <button 
              v-if="currentTab === 'releases'"
              @click="openReleaseModal"
              class="px-6 py-2 bg-amber-600 text-white rounded-lg hover:bg-amber-700 transition-colors shadow-sm font-medium"
            >
              Upload Release Manually
            </button>
            <button 
              v-else
              @click="openKeyModal"
              class="px-6 py-2 bg-stone-800 text-white rounded-lg hover:bg-stone-900 transition-colors shadow-sm font-medium"
            >
              Create New Key
            </button>
          </div>
        </div>

        <div v-if="loading" class="text-center py-20 text-stone-400">Loading...</div>

        <!-- Releases Table -->
        <div v-else-if="currentTab === 'releases'" class="bg-white rounded-2xl shadow-sm border border-stone-100 overflow-hidden">
             <!-- ... (Keep existing Releases Table) ... -->
             <div class="overflow-x-auto">
              <table class="w-full text-left text-sm">
                <thead>
                  <tr class="text-stone-500 border-b border-stone-100 bg-stone-50/50">
                    <th class="px-6 py-4 font-normal w-24">Status</th>
                    <th class="px-6 py-4 font-normal">Version</th>
                    <th class="px-6 py-4 font-normal">Description</th>
                    <th class="px-6 py-4 font-normal">File Info</th>
                    <th class="px-6 py-4 font-normal">Created At</th>
                    <th class="px-6 py-4 font-normal text-right">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-stone-50">
                  <tr v-for="release in releases" :key="release.id" class="hover:bg-stone-50/50" :class="{ 'bg-amber-50/30': isCurrent(release.id) }">
                    <td class="px-6 py-4">
                      <span v-if="isCurrent(release.id)" class="px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">Active</span>
                      <span v-else class="text-stone-400 text-xs">Archived</span>
                    </td>
                    <td class="px-6 py-4 font-medium text-stone-800">{{ release.version_name || 'Auto-Release' }}</td>
                    <td class="px-6 py-4 text-stone-600 truncate max-w-xs">{{ release.description || '-' }}</td>
                    <td class="px-6 py-4 text-xs text-stone-500">
                      <div class="font-medium text-stone-700">{{ release.file_original_name }}</div>
                      <div>{{ formatSize(release.file_size_bytes) }}</div>
                    </td>
                    <td class="px-6 py-4 text-stone-500">{{ formatDate(release.created_at) }}</td>
                    <td class="px-6 py-4 text-right flex justify-end gap-3 items-center">
                      <a 
                        :href="getDownloadUrl(release.file_id)"
                        class="text-stone-400 hover:text-amber-600 text-lg transition-colors"
                        title="Download DB"
                        :download="release.file_original_name"
                      >
                        ↓
                      </a>
                      <button 
                        v-if="!isCurrent(release.id)" 
                        @click="handleRollback(release)" 
                        class="text-amber-600 hover:text-amber-800 text-xs font-medium"
                      >
                        Rollback
                      </button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
        </div>

        <!-- Keys Table -->
        <div v-else class="bg-white rounded-2xl shadow-sm border border-stone-100 overflow-hidden">
          <div class="overflow-x-auto">
            <table class="w-full text-left text-sm">
              <thead>
                <tr class="text-stone-500 border-b border-stone-100 bg-stone-50/50">
                  <th class="px-6 py-4 font-normal">Status</th>
                  <th class="px-6 py-4 font-normal">Name</th>
                  <th class="px-6 py-4 font-normal">ID</th>
                  <th class="px-6 py-4 font-normal">Created At</th>
                  <th class="px-6 py-4 font-normal">Last Used</th>
                  <th class="px-6 py-4 font-normal text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-stone-50">
                <tr v-for="key in keys" :key="key.id" class="hover:bg-stone-50/50">
                  <td class="px-6 py-4">
                    <span v-if="key.is_active" class="px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">Active</span>
                    <span v-else class="px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">Revoked</span>
                  </td>
                  <td class="px-6 py-4 font-medium text-stone-800">{{ key.name || 'Unnamed' }}</td>
                  <td class="px-6 py-4 font-mono text-xs text-stone-400">{{ key.id }}</td>
                  <td class="px-6 py-4 text-stone-500">{{ formatDate(key.created_at) }}</td>
                  <td class="px-6 py-4 text-stone-500">{{ key.last_used_at ? formatDate(key.last_used_at) : 'Never' }}</td>
                  <td class="px-6 py-4 text-right">
                    <button 
                      @click="toggleKeyStatus(key)"
                      class="text-xs font-medium transition-colors"
                      :class="key.is_active ? 'text-red-600 hover:text-red-800' : 'text-green-600 hover:text-green-800'"
                    >
                      {{ key.is_active ? 'Revoke' : 'Activate' }}
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </main>

    <!-- Release Modal -->
    <div v-if="showReleaseModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm">
      <div class="bg-white rounded-2xl shadow-xl max-w-lg w-full p-8 relative">
        <h2 class="text-2xl font-light text-stone-800 mb-6">Create New Release</h2>
        <div class="space-y-6">
          <input ref="fileInput" type="file" accept=".db,.sqlite" @change="handleFileSelect" class="block w-full text-sm text-stone-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:bg-amber-50 file:text-amber-700 hover:file:bg-amber-100" />
          <input v-model="newRelease.versionName" type="text" placeholder="Version Name" class="w-full px-4 py-2 border border-stone-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500/50" />
          <textarea v-model="newRelease.description" rows="3" placeholder="Description" class="w-full px-4 py-2 border border-stone-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500/50"></textarea>
        </div>
        <div class="flex justify-end gap-3 mt-8">
          <button @click="closeReleaseModal" class="px-4 py-2 text-stone-500 hover:text-stone-800">Cancel</button>
          <button @click="submitRelease" class="px-6 py-2 bg-amber-600 text-white rounded-lg hover:bg-amber-700 disabled:opacity-50" :disabled="isUploading || !newRelease.file">{{ isUploading ? 'Uploading...' : 'Publish' }}</button>
        </div>
      </div>
    </div>

    <!-- Key Modal -->
    <div v-if="showKeyModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm">
      <div class="bg-white rounded-2xl shadow-xl max-w-md w-full p-8 relative">
        <h2 class="text-2xl font-light text-stone-800 mb-6">Create API Key</h2>
        
        <div v-if="!createdKeyResult">
          <div class="mb-6">
            <label class="block text-sm font-medium text-stone-600 mb-2">Key Name / Owner</label>
            <input v-model="newKeyName" type="text" placeholder="e.g. 'Production Client'" class="w-full px-4 py-2 border border-stone-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-500/20" autofocus @keyup.enter="submitKey" />
          </div>
          <div class="flex justify-end gap-3">
            <button @click="showKeyModal = false" class="px-4 py-2 text-stone-500 hover:text-stone-800">Cancel</button>
            <button @click="submitKey" class="px-6 py-2 bg-stone-800 text-white rounded-lg hover:bg-stone-900">Create</button>
          </div>
        </div>

        <div v-else class="text-center">
          <div class="mb-6">
            <div class="w-12 h-12 bg-green-100 text-green-600 rounded-full flex items-center justify-center mx-auto mb-4 text-xl">✓</div>
            <h3 class="text-lg font-medium text-stone-800 mb-2">Key Created!</h3>
            <p class="text-sm text-stone-500 mb-4">Copy this key now. You won't see it again.</p>
            <div class="bg-stone-50 border border-stone-200 p-4 rounded-lg font-mono text-sm break-all text-stone-800 select-all">
              {{ createdKeyResult.api_key }}
            </div>
          </div>
          <button @click="showKeyModal = false" class="w-full py-2 bg-stone-800 text-white rounded-lg hover:bg-stone-900">Done</button>
        </div>

      </div>
    </div>

  </div>
</template>
