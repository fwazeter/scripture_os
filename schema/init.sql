-- Enable hierarchical tree extension
CREATE EXTENSION IF NOT EXISTS ltree;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Enum for node types (Book, Chapter, Verse)
CREATE TYPE node_type_enum AS ENUM ('book', 'chapter', 'verse');

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

-- 4. Nodes (Structural Spine)
CREATE TABLE nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_id UUID REFERENCES works(id),
    path LTREE UNIQUE NOT NULL,
    node_type node_type_enum NOT NULL,
    sort_order DECIMAL NOT NULL
);

-- 5. Node Aliases (Resolves "Jn" -> "john")
CREATE TABLE node_aliases (
                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                              node_id UUID REFERENCES nodes(id),
                              alias TEXT NOT NULL,
                              is_canonical BOOLEAN DEFAULT FALSE,
                              UNIQUE(alias, node_id)
);

-- 6. Texts (The Content)
CREATE TABLE texts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID REFERENCES nodes(id),
    edition_id UUID REFERENCES editions(id),
    body_text TEXT NOT NULL
);

-- Create a GIST index for fast ltree traversal
CREATE INDEX idx_nodes_path ON nodes USING GIST (path);