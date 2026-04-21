This comprehensive plan for **Scripture OS (USDB V1)** integrates the original architectural goals with the critical improvements identified in the previous critiques. It transitions from a conceptual model to a production-ready blueprint.

---

## 1. Final Database Schema (The "Spine")

This schema utilizes PostgreSQL's `ltree` for hierarchy and incorporates the missing metadata and search optimizations required for a high-performance system.

### A. Taxonomy & Management
| Table | Description | Key Columns |
| :--- | :--- | :--- |
| **`traditions`** | Major faith categories. | `id`, `name` (unique), `metadata` (JSONB). |
| **`works`** | The scriptural corpus (e.g., Bible, Quran). | `id`, `tradition_id`, `slug` (unique), `title`. |
| **`editions`** | Specific translations/manuscripts. | `id`, `work_id`, `name`, `language_code`, `is_source` (bool), **`is_primary`** (bool - for default API view). |

### B. Structural Spine
| Table | Description | Key Columns |
| :--- | :--- | :--- |
| **`nodes`** | The "Address" of every verse/chapter. | `id`, `work_id`, **`path`** (LTREE), **`node_type`** (Enum: book/chap/verse), **`sort_order`** (DECIMAL for insertions). |
| **`node_aliases`**| Maps human input to paths. | `id`, `node_id`, `alias` (e.g., "Jn" -> "john"), `is_canonical` (bool). |

### C. Content & Search
| Table | Description | Key Columns |
| :--- | :--- | :--- |
| **`texts`** | The actual linguistic content. | `id`, `node_id`, `edition_id`, **`body_text`**, **`search_vector`** (TSVECTOR - generated). |

> **Unique Constraint:** `UNIQUE(node_id, edition_id)` ensures data integrity across translations.

---

## 2. Program Modules & Logic Layer

The program is divided into four specialized "Engines" that handle the lifecycle of a scriptural request.



### Module 1: Resolution Engine (The "Router")
This module handles the logic of finding the correct "Address ID" from a user-provided string.
* **`parseAddress(input_string)`**: Uses regex and the `node_aliases` table to convert "John 3:16" into the ltree path `bible.nt.john.3.16`.
* **`resolvePath(path_string)`**: Validates the ltree path against the `nodes` table and returns the `node_id`.
* **`getHierarchy(parent_path, depth)`**: Uses ltree operators (`<@`) to fetch all children (e.g., all verses in a chapter).

### Module 2: Content Engine (The "Assembler")
This module retrieves and formats the text.
* **`fetchText(node_ids[], edition_ids[])`**: Performs a JOIN between `texts`, `nodes`, and `editions`. If `edition_ids` is empty, it queries for `is_primary = TRUE` for the associated work.
* **`compareTranslations(node_id, edition_ids[])`**: Specifically optimized to return a side-by-side JSON structure for multiple translations of a single node.

### Module 3: Search Engine (The "Finder")
* **`searchKeyword(query, scope_path, edition_id)`**: Performs a Full-Text Search (FTS) using the `search_vector`.
* **`generateSnippets(query, text_ids[])`**: Uses PostgreSQL's `ts_headline` to return bolded search results in context.

### Module 4: Navigation Engine (The "Guide")
* **`getAdjacentNodes(current_node_id)`**: Uses the `sort_order` (Decimal) to find the immediately preceding and following nodes of the same `node_type` within the same work.

---

## 3. The Gateway Layer (API Endpoints)

The API orchestrates the modules above into public-facing REST or GraphQL endpoints.

* **`GET /v1/read/{path}?editions=id1,id2`**
    1. Call `parseAddress` then `resolvePath`.
    2. Call `fetchText` for the resolved ID.
* **`GET /v1/structure/{path}`**
    1. Call `getHierarchy` to return navigation menus (e.g., list of chapters in a book).
* **`GET /v1/search?q={query}&scope={path}`**
    1. Call `searchKeyword` restricted to the provided path.

---

## 4. Supporting Programs (Utilities)

A production Scripture OS requires two standalone programs outside of the main API.

### A. The Ingestion Pipeline (CLI Tool)
A dedicated program (likely written in Rust or Python) to seed the database.
* **`validateSchema(input_file)`**: Checks that the CSV/JSON matches the expected structure.
* **`ingestScriptureChunk(chunk)`**:
    1. Opens a database transaction.
    2. **Upsert Node**: Creates the path if it doesn't exist.
    3. **Upsert Text**: Inserts or updates the body text (handling typos/corrections).
    4. Commits transaction only on total success.

### B. The Versification Mapper
A specialized utility for handling the "Versification Trap".
* **`mapNodes(edition_a_node, edition_b_node)`**: Creates a manual override in a `versification_map` table to link verses that are numbered differently across traditions.

---

## 5. MVP Implementation Roadmap

1.  **Infrastructure**: Enable `ltree` and `pgcrypto`. Define the ENUMs for `node_type`.
2.  **Core Ingestion**: Import one "Source" text (e.g., SBLGNT) and one "Primary" translation (e.g., KJV) to establish the initial Structural Spine.
3.  **Basic API**: Implement `resolvePath` and `fetchText` to get a single verse via a URL.
4.  **UI/Navigation**: Implement `getHierarchy` to allow users to click through Books -> Chapters -> Verses.
5.  **Search**: Index the `texts` table and enable the `searchKeyword` function.