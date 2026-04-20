-- Enable hierarchical tree extension
CREATE EXTENSION IF NOT EXISTS ltree;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 1. Traditions (e.g. Abrahamic)
CREATE TABLE traditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL
);

-- 2. Works (e.g., The Bible, Quran)
CREATE TABLE works (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tradition_id UUID REFERENCES traditions(id),
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL
);

-- 3. Editions (e.g. KJV or Greek Source)
CREATE TABLE editions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_id UUID REFERENCES works(id),
    name TEXT NOT NULL,
    language_code VARCHAR(10) NOT NULL,
    is_source BOOLEAN DEFAULT FALSE
);

-- 4. Nodes (Structural Spine - Stand-off Markup)
CREATE TABLE nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_id UUID REFERENCES works(id) ON DELETE CASCADE,
    path LTREE UNIQUE NOT NULL,
    node_type VARCHAR(50) NOT NULL, --Allows for infinite flexibility like sura, chapter, book, section, etc
    start_index INTEGER NOT NULL, -- Where node begins in universal sequence
    end_index INTEGER NOT NULL -- where node ends in universal sequence
);

-- 5. Node Aliases (Resolves "Jn" -> "bible.nt.john")
CREATE TABLE node_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID REFERENCES nodes(id) ON DELETE CASCADE,
    alias TEXT NOT NULL,
    is_canonical BOOLEAN DEFAULT FALSE,
    UNIQUE(alias, node_id)
);

-- 6. Texts (The Content)
CREATE TABLE texts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_id UUID REFERENCES editions(id) ON DELETE CASCADE,
    absolute_index INTEGER NOT NULL, -- universal linear sequence number
    body_text TEXT NOT NULL,
    UNIQUE(edition_id, absolute_index) -- Prevents two texts occupying exact same slot in a single translation
);

-- Create a GIST index for fast ltree traversal
CREATE INDEX idx_nodes_path ON nodes USING GIST (path);
CREATE INDEX idx_nodes_work_indices ON nodes (work_id, start_index, end_index);
CREATE INDEX idx_texts_edition_index ON texts (edition_id, absolute_index);