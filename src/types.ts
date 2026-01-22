export type NoteType = 'thought' | 'harmony' | 'phrase' | 'rhythm' | 'score';

export interface NoteMetadata {
  duration?: number;
  time_signature?: string;
  mood?: string;
  file_path?: string;
  comment?: string;
}

export interface Note {
  id: string;
  note_type: NoteType;
  content: string;
  metadata: NoteMetadata;
  created_at: string;
  updated_at: string;
  is_public: boolean;
}

export interface CreateNoteRequest {
  note_type: NoteType;
  content: string;
  metadata?: NoteMetadata;
  is_public?: boolean;
}

export interface UpdateNoteRequest {
  content?: string;
  metadata?: NoteMetadata;
  is_public?: boolean;
}

export interface NotesFilter {
  note_type?: NoteType;
  year?: number;
  limit?: number;
  offset?: number;
}
