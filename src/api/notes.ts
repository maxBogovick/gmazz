import { invoke } from '@tauri-apps/api/core';
import type { Note, CreateNoteRequest, UpdateNoteRequest, NotesFilter } from '../types';

export async function createNote(request: CreateNoteRequest): Promise<Note> {
  return await invoke('create_note', { request });
}

export async function updateNote(id: string, request: UpdateNoteRequest): Promise<Note> {
  return await invoke('update_note', { id, request });
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
  return await invoke('delete_note', { id });
}

export async function uploadFile(
  fileName: string,
  fileData: number[],
  fileType: string
): Promise<string> {
  return await invoke('upload_file', {
    fileName,
    fileData,
    fileType,
  });
}

export async function getAssetPath(relativePath: string): Promise<string> {
  return await invoke('get_asset_path', { relativePath });
}
