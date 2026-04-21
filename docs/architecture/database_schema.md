# Architecture: Database Schema (The Physical Spine)

This document specifies the PostgreSQL schema for Scripture OS. The system utilizes an advanced **Sequential Range Mapping (Stand-off Markup)** model to decouple hierarchical traditions from linear linguistic content.

### **Architectural Design Decision: The Stand-off Markup Philosophy**
Traditional scripture databases often nest text within a specific hierarchy (e.g., a "Verse" table containing a `text` column). Scripture OS rejects this model in favor of a "Spine and Muscle" architecture:
1.  **The Spine (`nodes`)**: Manages addresses and boundaries using `ltree`.
2.  **The Muscle (`texts`)**: Manages the raw sequential text.

This decoupling allows a single physical verse to be part of infinite overlapping structural systems (e.g., different numbering traditions) without duplicating the text content.

---

## 1. System Requirements
The following PostgreSQL extensions must be enabled to support the core logic:
* **`ltree`**: Enables high-performance hierarchical path management and ancestor/descendant queries.
* **`pgcrypto`**: Used for secure, collision-resistant `gen_random_uuid()` functionality.

---

## 2. The Taxonomy Layer
Defines high-level categorization and ownership of scriptural works.

### **Table: `traditions`**
Groups works by their primary faith tradition (e.g., Abrahamic, Vedic).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `name` | TEXT | UNIQUE, NOT NULL | Common name of the tradition. |

### **Table: `works`**
Represents a conceptual scriptural corpus (e.g., "The Holy Bible").

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `tradition_id` | UUID | FOREIGN KEY | Links to `traditions(id)`. |
| `slug` | TEXT | UNIQUE, NOT NULL | Short code for URL routing (e.g., 'bible'). |
| `title` | TEXT | NOT NULL | The full descriptive title. |

---

## 3. The Structural Spine Layer
Manages the hierarchical organization and address resolution.

### **Table: `nodes`**
The core structural unit. Every node represents a "coordinate" in the library.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `work_id` | UUID | FOREIGN KEY | Links to `works(id)` with CASCADE. |
| `path` | LTREE | UNIQUE, NOT NULL | Hierarchical address (e.g., `bible.nt.john.3.16`). |
| `node_type` | VARCHAR | NOT NULL | Type identifier (e.g., 'sura', 'chapter', 'verse'). |
| `start_index`| INTEGER | NOT NULL | The inclusive beginning of the sequential range. |
| `end_index` | INTEGER | NOT NULL | The inclusive end of the sequential range. |

### **Table: `node_aliases`**
Provides a mapping between human-readable shorthand and canonical paths.

### **Architectural Design Decision: Interface Abstraction**
Resolving "Jn" to `bible.nt.john` is a data concern, not a hardcoded Rust concern. Aliases allow for multi-language support (e.g., "Génesis" and "Gen") without changing application logic.

---

## 4. The Content Layer
Stores linguistic text segments in a parallel sequence.

### **Table: `texts`**
The storage engine for every translation and edition.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `edition_id` | UUID | FOREIGN KEY | Links to `editions(id)` with CASCADE. |
| `absolute_index`| INTEGER | NOT NULL | The unique sequential slot for this segment. |
| `body_text` | TEXT | NOT NULL | The raw text content. |

**Constraint:** `UNIQUE(edition_id, absolute_index)` ensures structural integrity by preventing two text variants from occupying the same sequential slot within one translation.

---

## 5. Indexing & Optimization Strategy

### **Design Decision: High-Performance Discovery**
1.  **GIST Index (`nodes.path`)**: Essential for the `<@` (descendant) and `nlevel()` operators used in hierarchical navigation.
2.  **Composite B-Tree (`nodes(work_id, start_index, end_index)`)**: Optimized for the "Range Mapping" retrieval pattern, enabling the Content Engine to find text bounds instantly.
3.  **Composite B-Tree (`texts(edition_id, absolute_index)`)**: Bridges the spine back to physical content segments.

---

## 6. Logical Flow: Overlapping Traditions
### **Case Study: Hebrew vs. English Psalms**
* **The Problem:** English Bibles often treat the Psalm title as unnumbered metadata, while the Hebrew Tanakh counts it as Verse 1.
* **The Solution:** Both traditions point to the same linear sequence.
   * `bible.ot.psalm.51.title` -> maps to indices `1000` through `1001`.
   * `tanakh.ketuvim.psalm.51.1` -> maps to index `1000`.

**AI Prompt Hint:** When adding a new work with a unique numbering system, do not modify the `texts` table. Simply add new `nodes` with overlapping `start_index` and `end_index` values that point to the existing sequence.

