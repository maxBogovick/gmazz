use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NoteType {
    Thought,
    Harmony,
    Phrase,
    Rhythm,
    Score,
}

impl NoteType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NoteType::Thought => "thought",
            NoteType::Harmony => "harmony",
            NoteType::Phrase => "phrase",
            NoteType::Rhythm => "rhythm",
            NoteType::Score => "score",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "thought" => Some(NoteType::Thought),
            "harmony" => Some(NoteType::Harmony),
            "phrase" => Some(NoteType::Phrase),
            "rhythm" => Some(NoteType::Rhythm),
            "score" => Some(NoteType::Score),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mood: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl Default for NoteMetadata {
    fn default() -> Self {
        Self {
            duration: None,
            time_signature: None,
            mood: None,
            file_path: None,
            comment: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub note_type: NoteType,
    pub content: String,
    pub metadata: NoteMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub note_type: NoteType,
    pub content: String,
    #[serde(default)]
    pub metadata: NoteMetadata,
    #[serde(default = "default_true")]
    pub is_public: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub content: Option<String>,
    pub metadata: Option<NoteMetadata>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotesFilter {
    pub note_type: Option<NoteType>,
    pub year: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
