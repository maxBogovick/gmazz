-- settings: глобальные настройки сайта (профиль, тема, и т.д.)
-- Храним как key-value пары для гибкости
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,           -- уникальный ключ настройки
    value TEXT NOT NULL,            -- значение (JSON или строка)
    updated_at INTEGER NOT NULL     -- Unix timestamp последнего обновления
);

-- Индекс для быстрого поиска по времени обновления
CREATE INDEX IF NOT EXISTS idx_settings_updated_at ON settings(updated_at);
