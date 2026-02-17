CREATE INDEX IF NOT EXISTS idx_files_created_at ON files(created_at);
CREATE INDEX IF NOT EXISTS idx_archives_created_at ON archives(created_at);
CREATE INDEX IF NOT EXISTS idx_releases_is_active ON releases(is_active);
