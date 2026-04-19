# Scripture OS (USDB V1) Database Documentation

This document outlines the current database schema for the **Scripture OS** project. The architecture follows a **Layered Content** approach, separating the hierarchical "address" of a text from its actual linguistic content.

---

## 1. System Extensions
The following PostgreSQL extensions must be enabled:
* **`ltree`**: Used for hierarchical path management in the Structural Spine.
* **`pgcrypto`**: Used for `gen_random_uuid()` functionality.

---

## 2. Taxonomy Layer
*High-level categorization for the library.*

### Table: `traditions`
Defines the major faith traditions (e.g., Abrahamic, Dharmic).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `name` | TEXT | UNIQUE, NOT NULL | Common name (e.g., 'Abrahamic'). |
| `metadata` | JSONB | - | Optional flexible metadata. |

### Table: `works`
The "Master Book" or scriptural corpus.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `tradition_id` | UUID | FOREIGN KEY | Links to `traditions(id)`. |
| `slug` | TEXT | UNIQUE | Short code for pathing (e.g., 'bible'). |
| `title` | TEXT | NOT NULL | Full title (e.g., 'The Holy Bible'). |

---

## 3. Structural Spine Layer
*The "Where": Defines the address space of the scripture.*

### Table: `nodes`
Each row represents a unique location in the hierarchy (Book, Chapter, or Verse).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | The "Address ID" for all translations. |
| `work_id` | UUID | FOREIGN KEY | Links to `works(id)`. |
| `path` | **LTREE** | UNIQUE, NOT NULL | Hierarchical path (e.g., `bible.nt.jn.3.16`). |
| `sort_order` | INTEGER | NOT NULL | Used for sequential UI rendering. |

---

## 4. Content Layer
*The "What": Stores the actual text for various editions.*

### Table: `editions`
Represents a specific translation or a source manuscript critical edition.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `work_id` | UUID | FOREIGN KEY | Links to `works(id)`. |
| `name` | TEXT | UNIQUE, NOT NULL | e.g., 'King James Version' or 'SBLGNT'. |
| `language_code` | VARCHAR(10)| NOT NULL | ISO code (e.g., 'en', 'grc', 'heb'). |
| `is_source` | BOOLEAN | DEFAULT FALSE | TRUE for original language manuscripts. |

### Table: `texts`
The actual strings of scripture, linked to an address (`node`) and a version (`edition`).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `node_id` | UUID | FOREIGN KEY | Links to `nodes(id)`. |
| `edition_id` | UUID | FOREIGN KEY | Links to `editions(id)`. |
| `body_text` | TEXT | NOT NULL | The readable scripture string. |

**Composite Constraints:**
* `UNIQUE(node_id, edition_id)`: Prevents duplicate translations for the same verse.

---

## 5. Indexing Strategy
To maintain performance in the Rust API, the following indexes are applied:

1.  **GIST Index (`nodes.path`)**: Critical for the `<@` (descendant) and `~` (match) operators used in hierarchical fetching.
2.  **B-Tree Index (`nodes.sort_order`)**: Ensures rapid sequential retrieval for chapter/book views.
3.  **GIN Index (`texts.body_text`)**: (Planned) To enable high-speed Full-Text Search (FTS).

---

## 6. Logical Flow (Example)
To retrieve **John 3:16** in both **English** and **Greek**:

1.  The API receives the path `bible.nt.jn.3.16`.
2.  The `nodes` table is queried to find the `node_id` for that path.
3.  The `texts` table is queried for all rows matching that `node_id`.
4.  The results are joined with the `editions` table to provide the user with:
    * **KJV**: "For God so loved..."
    * **SBLGNT**: "Οὕτως γὰρ ἠγάπησεν..."

---