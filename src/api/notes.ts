import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';
import type { Note, CreateNoteRequest, UpdateNoteRequest, NotesFilter } from '../types';
import { SERVER_URL } from './server';

// Simple detection of Tauri environment

export async function createNote(request: CreateNoteRequest): Promise<Note> {
    if (!isTauri) throw new Error('Action not supported in Web Mode');
    const note = await invoke<Note>('create_note', { request });
    // syncDatabase removed to allow offline-first / manual sync
    return note;
}

export async function updateNote(id: string, request: UpdateNoteRequest): Promise<Note> {
    if (!isTauri) throw new Error('Action not supported in Web Mode');
    const note = await invoke<Note>('update_note', { id, request });
    // syncDatabase removed
    return note;
}

export async function getNotes(filter?: NotesFilter): Promise<Note[]> {
    if (isTauri()) {
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
    if (isTauri()) {
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
    if (isTauri()) {
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
    // syncDatabase removed
}

export async function uploadFile(
    file: File
): Promise<string> {
    if (!isTauri) throw new Error('Action not supported in Web Mode');

    const arrayBuffer = await file.arrayBuffer();
    const bytes = Array.from(new Uint8Array(arrayBuffer));

    return await invoke<string>('upload_file', {
        fileName: file.name,
        fileData: bytes,
        fileType: file.type
    });
}

import { getApiKey, isTauri } from './server';

export async function getAssetPath(relativePath: string): Promise<string> {
    if (isTauri()) {
        if (relativePath.startsWith('http')) return relativePath;
        try {
            const res = await invoke<{ path: string, mime_type: string | null }>('get_asset_path', { relativePath });
            const data = await readFile(res.path);
            const blob = new Blob([data], { type: res.mime_type || undefined });
            return URL.createObjectURL(blob);
        } catch (e) {
            console.error('Failed to load local asset:', e);
            return relativePath; // Fallback
        }
    } else {
        // Web Mode
        let url = relativePath;

        if (!url.startsWith('http')) {
            url = `${SERVER_URL}/v1/files/${relativePath}`;
        }

        try {
            const apiKey = getApiKey();
            const headers: Record<string, string> = {};
            if (apiKey) {
                headers['X-API-KEY'] = apiKey;
            }

            const response = await fetch(url, { headers });
            if (!response.ok) throw new Error(`Failed to fetch asset: ${response.statusText}`);

            const blob = await response.blob();
            return URL.createObjectURL(blob);
        } catch (e) {
            console.error('Asset fetch failed:', e);
            return url; // Fallback
        }
    }
}

export async function syncDatabase(): Promise<string> {
    if (!isTauri) throw new Error('Sync not supported in Web Mode');
    const apiKey = getApiKey();
    return await invoke('sync_local_db_to_server', { apiKey });
}
