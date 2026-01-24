-- releases: история версий базы данных
CREATE TABLE releases (
  id TEXT PRIMARY KEY,            -- uuid
  app_id TEXT NOT NULL,
  file_id TEXT NOT NULL,          -- ссылка на физический файл
  version_name TEXT,              -- "Auto-backup", "v1.0.0", etc
  description TEXT,
  is_active INTEGER DEFAULT 0,    -- не используется если мы просто берем последнюю по дате, но полезно для "Pinning"
  created_at INTEGER NOT NULL,
  FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE,
  FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_releases_created_at ON releases(created_at);
