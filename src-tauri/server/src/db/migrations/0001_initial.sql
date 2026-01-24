-- apps: приложения / владельцы ключей
CREATE TABLE apps (
  id TEXT PRIMARY KEY,            -- uuid
  api_key_hash TEXT UNIQUE NOT NULL,
  name TEXT,
  is_active INTEGER DEFAULT 1,
  created_at INTEGER NOT NULL,    -- unix epoch
  last_used_at INTEGER            -- nullable
);

-- files: мета по файлам
CREATE TABLE files (
  id TEXT PRIMARY KEY,            -- uuid
  app_id TEXT NOT NULL,
  original_name TEXT NOT NULL,
  stored_path TEXT NOT NULL,      -- относительный путь внутри ./data
  mime_type TEXT,
  size_bytes INTEGER NOT NULL,
  checksum TEXT,                  -- sha256 hex
  created_at INTEGER NOT NULL,
  deleted_at INTEGER,             -- nullable, soft-delete
  FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

-- archives: сгенерированные архивы
CREATE TABLE archives (
  id TEXT PRIMARY KEY,
  app_id TEXT NOT NULL,
  status TEXT NOT NULL,           -- pending / ready / failed
  file_path TEXT,                 -- относительный путь к zip
  created_at INTEGER NOT NULL,
  expires_at INTEGER,
  FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

-- индексы
CREATE INDEX idx_files_app ON files(app_id);
CREATE INDEX idx_files_deleted_at ON files(deleted_at);
CREATE INDEX idx_archives_app ON archives(app_id);
