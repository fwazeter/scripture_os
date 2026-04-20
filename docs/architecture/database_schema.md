# Scripture OS (USDB V1) Database Documentation

This document outlines the database schema for the **Scripture OS** project. The architecture utilizes an advanced **Sequential Range Mapping (Stand-off Markup)** model. This separates the hierarchical "address" or "tradition" of a text from its continuous linear sequence, allowing for infinite, overlapping organizational structures without duplicating text.

---

## 1. System Extensions
The following PostgreSQL extensions must be enabled:
* **`ltree`**: Used for hierarchical path management in the Structural Maps.
* **`pgcrypto`**: Used for `gen_random_uuid()` functionality.

---

## 2. Taxonomy Layer
*High-level categorization for the library.*

### Table: `traditions`
Defines the major faith traditions (e.g., Abrahamic, Vedic).

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `name` | TEXT | UNIQUE, NOT NULL | Common name (e.g., 'Abrahamic'). |

### Table: `works`
The "Master Book" or conceptual scriptural corpus.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `tradition_id` | UUID | FOREIGN KEY | Links to `traditions(id)`. |
| `slug` | TEXT | UNIQUE | Short code for routing (e.g., 'bible'). |
| `title` | TEXT | NOT NULL | Full title (e.g., 'The Holy Bible'). |

---

## 3. Structural Spine Layer (The Range Maps)
*The "Where": Defines the address overlays using `ltree` and range pointers.*

### Table: `nodes`
Each row represents a logical structural boundary (Book, Sura, Chapter, Verse, Sloka). It does NOT hold text, but points to a sequence range.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | The node identifier. |
| `work_id` | UUID | FOREIGN KEY | Links to `works(id)`. CASCADES. |
| `path` | **LTREE** | UNIQUE, NOT NULL | Hierarchical path (e.g., `hafs.sura.1.1`). |
| `node_type`| VARCHAR(50)| NOT NULL | Dynamic string (e.g., 'book', 'ayah', 'superscription'). |
| `start_index`| INTEGER | NOT NULL | Where this structural node begins in the universal sequence. |
| `end_index` | INTEGER | NOT NULL | Where this structural node ends. |

### Table: `node_aliases`
Maps human shorthand to canonical paths.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `node_id` | UUID | FOREIGN KEY | Links to `nodes(id)`. CASCADES. |
| `alias` | TEXT | NOT NULL | e.g., "Jn" or "Gen". |
| `is_canonical`| BOOLEAN | DEFAULT FALSE | Is this the preferred alias? |

**Constraint:** `UNIQUE(alias, node_id)`

---

## 4. Content Layer (The Universal Sequence)
*The "What": Stores the actual text attached directly to an absolute linear number.*

### Table: `editions`
Represents a specific translation or source manuscript.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `work_id` | UUID | FOREIGN KEY | Links to `works(id)`. |
| `name` | TEXT | NOT NULL | e.g., 'KJV' or 'Hafs_Arabic'. |
| `language_code` | VARCHAR(10)| NOT NULL | ISO code (e.g., 'en', 'ar'). |
| `is_source` | BOOLEAN | DEFAULT FALSE | TRUE for original language manuscripts. |

### Table: `texts`
The continuous string of text. Decoupled from structure.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUID | PRIMARY KEY | Unique identifier. |
| `edition_id` | UUID | FOREIGN KEY | Links to `editions(id)`. CASCADES. |
| `absolute_index`| INTEGER | NOT NULL | The universal sequence number for this work (0 to N). |
| `body_text` | TEXT | NOT NULL | The readable scripture string. |

**Constraint:** `UNIQUE(edition_id, absolute_index)` prevents two texts occupying the exact same sequential slot in a single translation.

---

## 5. Indexing Strategy
1. **GIST Index (`nodes.path`)**: Critical for the `<@` operator used in hierarchical tree fetching.
2. **Composite B-Tree (`nodes(work_id, start_index, end_index)`)**: Enables blazing fast BETWEEN lookups for range mapping.
3. **Composite B-Tree (`texts(edition_id, absolute_index)`)**: Instantly links the requested range back to the physical text.

---

## 6. Logical Flow (Example: Hafs vs. Warsh)
To retrieve the first verse of the Quran:
1. The API receives a request for path `warsh.sura.1.1`.
2. The DB queries `nodes` and sees `warsh.sura.1.1` maps to `start_index: 2001` and `end_index: 2001` (skipping the unnumbered Basmala).
3. The DB queries `texts` for all rows `BETWEEN 2001 AND 2001`.
4. It returns "Alhamdulillah..."
   *(If the user asked for `hafs.sura.1.1`, it would see the index maps to `2000` and return the Basmala instead, without duplicating any text!).*