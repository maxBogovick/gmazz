// @ts-ignore
import { invoke } from '@tauri-apps/api/core';

export const SERVER_URL = import.meta.env.VITE_SERVER_URL || 'http://localhost:8080';
export const isTauri = () => !!(window as any).__TAURI_INTERNALS__;

let currentApiKey = import.meta.env.VITE_API_KEY || '';
if (!isTauri()) {
  const stored = localStorage.getItem('gmazz_api_key');
  if (stored) currentApiKey = stored;
}

export const getApiKey = () => currentApiKey;
export const setApiKey = (key: string) => {
  currentApiKey = key;
  if (!isTauri()) {
    localStorage.setItem('gmazz_api_key', key);
  }
};

// Deprecated export for backward compatibility if used directly elsewhere, but try to use getApiKey()
export const API_KEY = currentApiKey;

export interface Release {
  id: string;
  file_id: string;
  version_name: string | null;
  description: string | null;
  created_at: number;
  file_original_name: string;
  file_size_bytes: number;
}

export async function getReleases(limit = 20, offset = 0): Promise<Release[]> {
  if (isTauri()) throw new Error('Releases API is disabled in offline mode');
  const key = getApiKey();
  if (!key) throw new Error('API Key is not configured');

  const response = await fetch(`${SERVER_URL}/v1/releases?limit=${limit}&offset=${offset}`, {
    headers: { 'X-API-KEY': key }
  });

  if (!response.ok) throw new Error(`Failed to fetch releases: ${response.statusText}`);
  return await response.json();
}

export async function getLatestRelease(): Promise<Release | null> {
  if (isTauri()) throw new Error('Releases API is disabled in offline mode');
  const key = getApiKey();
  if (!key) throw new Error('API Key is not configured');

  const response = await fetch(`${SERVER_URL}/v1/releases/latest`, {
    headers: { 'X-API-KEY': key }
  });

  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Failed to fetch latest release: ${response.statusText}`);
  return await response.json();
}

export interface AppKey {
  id: string;
  name: string | null;
  is_active: boolean;
  created_at: number;
  last_used_at: number | null;
}

export interface CreatedKey {
  id: string;
  name: string;
  api_key: string;
}

export async function listKeys(adminSecret: string): Promise<AppKey[]> {
  if (isTauri()) throw new Error('Keys API is disabled in offline mode');
  const response = await fetch(`${SERVER_URL}/admin/keys`, {
    headers: { 'X-ADMIN-SECRET': adminSecret }
  });

  if (!response.ok) throw new Error(`Failed to list keys: ${response.statusText}`);
  return await response.json();
}

export async function createGuestKey(): Promise<CreatedKey> {
  if (isTauri()) throw new Error('Keys API is disabled in offline mode');
  const response = await fetch(`${SERVER_URL}/v1/public/auth/guest`, {
    method: 'POST'
  });

  if (!response.ok) throw new Error(`Failed to create guest key: ${response.statusText}`);
  return await response.json();
}

export async function createKey(adminSecret: string, name: string): Promise<CreatedKey> {
  if (isTauri()) throw new Error('Keys API is disabled in offline mode');
  const response = await fetch(`${SERVER_URL}/admin/keys`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-ADMIN-SECRET': adminSecret
    },
    body: JSON.stringify({ name })
  });

  if (!response.ok) throw new Error(`Failed to create key: ${response.statusText}`);
  return await response.json();
}

export async function updateKeyStatus(adminSecret: string, id: string, active: boolean): Promise<void> {
  if (isTauri()) throw new Error('Keys API is disabled in offline mode');
  const action = active ? 'activate' : 'revoke';
  const response = await fetch(`${SERVER_URL}/admin/keys/${id}/${action}`, {
    method: 'PUT',
    headers: { 'X-ADMIN-SECRET': adminSecret }
  });

  if (!response.ok) throw new Error(`Failed to ${action} key: ${response.statusText}`);
}

export interface CreateReleaseRequest {
  file_id: string;
  version_name?: string;
  description?: string;
}

export async function createRelease(request: CreateReleaseRequest): Promise<Release> {
  if (isTauri()) throw new Error('Release upload is disabled in offline mode');
  const key = getApiKey();
  if (!key) throw new Error('API Key is not configured');

  const response = await fetch(`${SERVER_URL}/v1/releases`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-KEY': key
    },
    body: JSON.stringify(request)
  });

  if (!response.ok) throw new Error(`Failed to create release: ${response.statusText}`);
  return await response.json();
}

export async function uploadFileToServer(file: File): Promise<string> {
  if (isTauri()) {
    // Use Local Desktop Command
    const arrayBuffer = await file.arrayBuffer();
    const bytes = Array.from(new Uint8Array(arrayBuffer));

    const id = await invoke('upload_file', {
      fileName: file.name,
      fileData: bytes,
      fileType: file.type
    }) as string;

    // Return ID (or asset URL if we decide later)
    // Note: If the backend returns just ID, the frontend needs to know how to resolve it.
    // But let's assume `asset://` handling or similar is handled by getAssetPath or something.
    // The current prompt asks for "save to filesystem", which we did.
    return id;
  }

  const key = getApiKey();
  if (!key) {
    throw new Error('API Key is not configured');
  }

  // The Rust server implementation (src-tauri/server/src/api/files.rs) uses `Body` stream and header `X-File-Name`
  // It expects RAW BINARY body, NOT multipart/form-data.

  const headers = {
    'X-API-KEY': key,
    'X-File-Name': file.name,
    'Content-Type': file.type || 'application/octet-stream'
  };

  const response = await fetch(`${SERVER_URL}/v1/files`, {
    method: 'POST',
    headers: headers,
    body: file
  });

  if (!response.ok) {
    throw new Error(`Upload failed: ${response.statusText}`);
  }

  const data = await response.json();
  // We return the FULL URL for assets, but for DB logic we might need just ID.
  // However, the existing contract for notes returns URL.
  // Let's keep this as URL returner. 
  // We'll create a helper for just ID or extract it.
  return `${SERVER_URL}/v1/files/${data.id}`;
}

export async function uploadFileAndGetId(file: File): Promise<string> {
  const url = await uploadFileToServer(file);
  return url.split('/').pop() || '';
}

// --- Settings API ---

export interface Setting {
  key: string;
  value: string;
  updated_at: number;
}

export interface AllSettingsResponse {
  settings: Setting[];
}

export async function getAllSettings(): Promise<Setting[]> {
  if (isTauri()) {
    return await invoke<Setting[]>('get_all_settings');
  }
  const response = await fetch(`${SERVER_URL}/v1/public/settings`);
  if (!response.ok) throw new Error(`Failed to fetch settings: ${response.statusText}`);
  const data: AllSettingsResponse = await response.json();
  return data.settings;
}

export async function getSetting(key: string): Promise<string | null> {
  if (isTauri()) {
    return await invoke<string | null>('get_setting', { key });
  }
  const response = await fetch(`${SERVER_URL}/v1/public/settings/${encodeURIComponent(key)}`);
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Failed to fetch setting: ${response.statusText}`);
  const data: Setting = await response.json();
  return data.value;
}

export async function setSetting(key: string, value: string): Promise<Setting> {
  if (isTauri()) {
    return await invoke<Setting>('set_setting', { key, value });
  }
  const apiKey = getApiKey();
  if (!apiKey) throw new Error('API Key is not configured');

  const response = await fetch(`${SERVER_URL}/v1/settings/${encodeURIComponent(key)}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
      'X-API-KEY': apiKey
    },
    body: JSON.stringify({ value })
  });

  if (!response.ok) throw new Error(`Failed to set setting: ${response.statusText}`);
  return await response.json();
}

export async function deleteSetting(key: string): Promise<boolean> {
  if (isTauri()) {
    return await invoke<boolean>('delete_setting', { key });
  }
  const apiKey = getApiKey();
  if (!apiKey) throw new Error('API Key is not configured');

  const response = await fetch(`${SERVER_URL}/v1/settings/${encodeURIComponent(key)}`, {
    method: 'DELETE',
    headers: {
      'X-API-KEY': apiKey
    }
  });

  if (response.status === 404) return false;
  if (!response.ok) throw new Error(`Failed to delete setting: ${response.statusText}`);
  return true;
}
