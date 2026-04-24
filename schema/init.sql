-- ==========================================
-- SCRIPTURE OS: DATABASE SCHEMA V1
-- ==========================================

-- 1. The Universal Dictionary (The "What")
-- Stores deduplicated strings of text.
CREATE TABLE IF NOT EXISTS fsi_lexicon
(
    id        BIGSERIAL PRIMARY KEY,
    body_text TEXT NOT NULL UNIQUE
);

-- 2. The Structural Spine (The "Where")
-- Maps a 3D Coordinate to a specific text inside the Dictionary.
CREATE TABLE IF NOT EXISTS fsi_scroll
(
    work_id      INTEGER  NOT NULL,
    macro_id     INTEGER  NOT NULL,
    lex_key      TEXT     NOT NULL,
    namespace_id INTEGER  NOT NULL,
    lexicon_id   BIGINT   NOT NULL REFERENCES fsi_lexicon (id),
    sub_mask     SMALLINT NOT NULL,
    merkle_hash  BYTEA    NOT NULL,
    PRIMARY KEY (work_id, macro_id, lex_key, namespace_id)
);

-- 3. The Resolution Map (The "Router")
-- Allows human shorthands to be routed to pure mathematical coordinates.
CREATE TABLE IF NOT EXISTS fsi_aliases
(
    alias    TEXT PRIMARY KEY,
    work_id  INTEGER NOT NULL,
    macro_id INTEGER NOT NULL,
    lex_key  TEXT    NOT NULL
);

-- ==========================================
-- OPTIMIZATION INDEXES
-- ==========================================

-- Speeds up the `get_next_atom` Traversal Engine queries.
CREATE INDEX IF NOT EXISTS idx_fsi_scroll_traversal
    ON fsi_scroll (work_id, macro_id, lex_key);

-- Speeds up translation lookups for side-by-side comparisons.
CREATE INDEX IF NOT EXISTS idx_fsi_scroll_coordinate
    ON fsi_scroll (work_id, macro_id, lex_key);