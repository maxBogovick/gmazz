import { invoke } from '@tauri-apps/api/core';
import type { Note, CreateNoteRequest, UpdateNoteRequest, NotesFilter } from '../types';
import { uploadFileToServer } from './server';

export async function createNote(request: CreateNoteRequest): Promise<Note> {
  const note = await invoke<Note>('create_note', { request });
  await syncDatabase();
  return note;
}

export async function updateNote(id: string, request: UpdateNoteRequest): Promise<Note> {
  const note = await invoke<Note>('update_note', { id, request });
  await syncDatabase();
  return note;
}

export async function getNotes(filter?: NotesFilter): Promise<Note[]> {
  return await invoke('get_notes', { filter });
}

export async function getNote(id: string): Promise<Note> {
  return await invoke('get_note', { id });
}

export async function getRandomNote(): Promise<Note> {
  return await invoke('get_random_note');
}

export async function deleteNote(id: string): Promise<void> {
  await invoke('delete_note', { id });
  await syncDatabase();
}

export async function uploadFile(
  file: File
): Promise<string> {
    return await uploadFileToServer(file);
}

// Deprecated: kept for signature compatibility if needed, but we changed the signature above
// logic to use the browser File object which is cleaner for frontend
// If the caller passes raw bytes, we might need adapter, but standard input is <input type="file">

export async function getAssetPath(relativePath: string): Promise<string> {
  // If it's already a URL, return it
  if (relativePath.startsWith('http')) {
      return relativePath;
  }
  return await invoke('get_asset_path', { relativePath });
}

export async function syncDatabase(): Promise<string> {
    try {
        return await invoke('sync_local_db_to_server');
    } catch (e) {
        console.warn('Background sync failed:', e);
        return '';
    }
}
