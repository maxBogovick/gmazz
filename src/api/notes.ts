import { invoke } from '@tauri-apps/api/core';
import type { Note, CreateNoteRequest, UpdateNoteRequest, NotesFilter } from '../types';
import { uploadFileToServer, SERVER_URL } from './server';

// Simple detection of Tauri environment
const isTauri = !!(window as any).__TAURI_INTERNALS__;

export async function createNote(request: CreateNoteRequest): Promise<Note> {
  if (!isTauri) throw new Error('Action not supported in Web Mode');
  const note = await invoke<Note>('create_note', { request });
  await syncDatabase();
  return note;
}

export async function updateNote(id: string, request: UpdateNoteRequest): Promise<Note> {
  if (!isTauri) throw new Error('Action not supported in Web Mode');
  const note = await invoke<Note>('update_note', { id, request });
  await syncDatabase();
  return note;
}

export async function getNotes(filter?: NotesFilter): Promise<Note[]> {
  if (isTauri) {
    return await invoke('get_notes', { filter });
  } else {
    // Web Mode: Fetch from Public API
    const params = new URLSearchParams();
    if (filter?.limit) params.append('limit', filter.limit.toString());
    if (filter?.offset) params.append('offset', filter.offset.toString());
    // Note: Public API currently returns only public notes, filtering by type/year not yet impl on backend public API but we can add later.
    // For now we just fetch list.
    
    const response = await fetch(`${SERVER_URL}/v1/public/notes?${params.toString()}`);
    if (!response.ok) throw new Error('Failed to fetch public notes');
    
    // Transform backend PublicNote (string dates) to Note (Date objects expected by UI?) 
    // Actually types.ts defines string or Date? Let's check.
    // Usually deserialization in JS keeps them as strings unless manually converted.
    const notes = await response.json();
    return notes.map((n: any) => ({
        ...n,
        metadata: typeof n.metadata === 'string' ? JSON.parse(n.metadata) : n.metadata
    }));
  }
}

export async function getNote(id: string): Promise<Note> {
  if (isTauri) {
    return await invoke('get_note', { id });
  } else {
    const response = await fetch(`${SERVER_URL}/v1/public/notes/${id}`);
    if (!response.ok) throw new Error('Failed to fetch note');
    const note = await response.json();
    if (typeof note.metadata === 'string') {
        note.metadata = JSON.parse(note.metadata);
    }
    return note;
  }
}

export async function getRandomNote(): Promise<Note> {
  if (isTauri) {
    return await invoke('get_random_note');
  } else {
    // Implement random on public API or just pick one from list?
    // For MVP Web, maybe just disabled or picking random locally from a list fetch?
    // Let's throw for now or fetch list and pick one.
    throw new Error('Random note not implemented for Web yet');
  }
}

export async function deleteNote(id: string): Promise<void> {
  if (!isTauri) throw new Error('Action not supported in Web Mode');
  await invoke('delete_note', { id });
  await syncDatabase();
}

export async function uploadFile(
  file: File
): Promise<string> {
    if (!isTauri) throw new Error('Action not supported in Web Mode');
    return await uploadFileToServer(file);
}

import { getApiKey } from './server';

export async function getAssetPath(relativePath: string): Promise<string> {
  if (isTauri) {
    if (relativePath.startsWith('http')) return relativePath;
    return await invoke('get_asset_path', { relativePath });
  } else {
    // Web Mode
    let url = relativePath;
    
    // If it's a legacy local path, we might assume it's served by the server somehow?
    // But current architecture stores http links in DB.
    // If we have a http link:
    if (url.startsWith('http')) {
       // We fetch it with the header and create a blob URL
       try {
           const apiKey = getApiKey();
           const headers: Record<string, string> = {};
           if (apiKey) {
               headers['GUEST_API_KEY'] = apiKey;
           }
           
           const response = await fetch(url, { headers });
           if (!response.ok) throw new Error(`Failed to fetch asset: ${response.statusText}`);
           
           const blob = await response.blob();
           return URL.createObjectURL(blob);
       } catch (e) {
           console.error('Asset fetch failed:', e);
           return url; // Fallback to raw URL (will likely fail 401 but better than crash)
       }
    }
    
    return url;
  }
}

export async function syncDatabase(): Promise<string> {
    if (!isTauri) return '';
    try {
        return await invoke('sync_local_db_to_server');
    } catch (e) {
        console.warn('Background sync failed:', e);
        return '';
    }
}
