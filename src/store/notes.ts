import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Note, NoteType, CreateNoteRequest, UpdateNoteRequest, NotesFilter } from '../types';
import * as api from '../api/notes';

export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([]);
  const currentNote = ref<Note | null>(null);
  const loading = ref(false);
  const saving = ref(false);
  const filter = ref<NotesFilter>({});
  const hasMore = ref(true);
  const LIMIT = 12;

  const filteredNotes = computed(() => {
    let result = [...notes.value];

    // Client-side filtering is still useful for immediate UI updates, 
    // but the primary source of truth for the list is now the paginated API response.
    // However, if we filter by type/year strictly on client side while paginating, 
    // it might behave oddly (showing gaps). 
    // For now, we rely on the API filter. 
    // If the user sets a filter, we reload the list from the server.
    
    // We can keep specific client-side sorts if needed, 
    // but default API sort should be respected.
    return result; 
  });

  async function fetchNotes(newFilter?: NotesFilter, append: boolean = false) {
    if (loading.value) return;
    
    loading.value = true;
    try {
      if (newFilter) {
        filter.value = { ...newFilter };
        // Reset for new filter
        if (!append) {
            notes.value = [];
            hasMore.value = true;
        }
      }

      // If we are not appending (e.g. initial load or refresh), clear list
      if (!append) {
          notes.value = [];
          hasMore.value = true;
      }

      const offset = notes.value.length;
      const apiFilter: NotesFilter = {
        ...filter.value,
        limit: LIMIT,
        offset: offset
      };

      const fetchedNotes = await api.getNotes(apiFilter);

      if (fetchedNotes.length < LIMIT) {
        hasMore.value = false;
      }

      if (append) {
        notes.value.push(...fetchedNotes);
      } else {
        notes.value = fetchedNotes;
      }
      
    } catch (error) {
      console.error('Failed to fetch notes:', error);
    } finally {
      loading.value = false;
    }
  }

  async function loadMore() {
    if (hasMore.value && !loading.value) {
      await fetchNotes(undefined, true);
    }
  }

  async function fetchNote(id: string) {
    loading.value = true;
    try {
      currentNote.value = await api.getNote(id);
    } catch (error) {
      console.error('Failed to fetch note:', error);
    } finally {
      loading.value = false;
    }
  }

  async function fetchRandomNote() {
    loading.value = true;
    try {
      currentNote.value = await api.getRandomNote();
      return currentNote.value;
    } catch (error) {
      console.error('Failed to fetch random note:', error);
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function createNote(request: CreateNoteRequest): Promise<Note | null> {
    saving.value = true;
    try {
      const note = await api.createNote(request);
      notes.value.unshift(note);
      currentNote.value = note;
      return note;
    } catch (error) {
      console.error('Failed to create note:', error);
      return null;
    } finally {
      saving.value = false;
    }
  }

  async function updateNote(id: string, request: UpdateNoteRequest): Promise<Note | null> {
    saving.value = true;
    try {
      const note = await api.updateNote(id, request);
      const index = notes.value.findIndex(n => n.id === id);
      if (index !== -1) {
        notes.value[index] = note;
      }
      if (currentNote.value?.id === id) {
        currentNote.value = note;
      }
      return note;
    } catch (error) {
      console.error('Failed to update note:', error);
      return null;
    } finally {
      saving.value = false;
    }
  }

  async function deleteNote(id: string): Promise<boolean> {
    try {
      await api.deleteNote(id);
      notes.value = notes.value.filter(n => n.id !== id);
      if (currentNote.value?.id === id) {
        currentNote.value = null;
      }
      return true;
    } catch (error) {
      console.error('Failed to delete note:', error);
      return false;
    }
  }

  async function setFilter(noteType?: NoteType, year?: number) {
    filter.value = { note_type: noteType, year };
    await fetchNotes({ note_type: noteType, year });
  }

  function clearFilter() {
    filter.value = {};
  }

  function clearCurrentNote() {
    currentNote.value = null;
  }

  return {
    notes,
    currentNote,
    loading,
    saving,
    filter,
    hasMore,
    filteredNotes,
    fetchNotes,
    loadMore,
    fetchNote,
    fetchRandomNote,
    createNote,
    updateNote,
    deleteNote,
    setFilter,
    clearFilter,
    clearCurrentNote,
  };
});
