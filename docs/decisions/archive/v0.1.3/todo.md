# Scripture OS: Development Plan v0.1.3 to v0.1.4

## 📂 Status Summary
* **Archived:** Phase 1 & 2 (Comparison Logic, Basic Search, Utility Versification Mapper) are 100% complete and moved to `docs/decisions/archive/`.
* **Active Focus:** Transitioning the "Physical Spine" to a **Fractal Semantic Index** and abstracting Engine logic for **WASM Plugin** support.

---

## 🟥 High Priority: The Hybrid Scripture Stack (Layer 1 & 2)
The primary goal is to move from a static integer-based sequence to a high-precision, track-aware decimal sequence.

### 1. Implement Fractal Semantic Indexing (FSI)
* [ ] **Database Migration:** Alter `texts` and `nodes` tables to use `NUMERIC(20,10)` for `absolute_index`.
* [ ] **Sequence Slot Management:** Create the `sequence_slots` table to provide immutable identities (UUIDs) for every atomized decimal coordinate.
* [ ] **Rust Model Refactor:** Update `ScriptureContent` and `HierarchyNode` in `src/models.rs` to support decimal-based indexing.
* [ ] **Ingestion Pipeline Update:** Update the internal seeder in `src/test_utils.rs` to support word-level atomization of `quran-uthmani.txt`.

### 2. WASM-Ready Engine Refactoring (Layer 3)
Abstracting the engines to host external logic "Lenses".
* [ ] **Engine Plugin Trait:** Define the `PluginHost` trait in `src/engines/mod.rs` to allow dynamic loading of WASM logic modules.
* [ ] **Resolution Engine Decoupling:** Refactor `CoreResolutionEngine` to delegate shorthand resolution to a loaded plugin instead of hard-coded regex.
* [ ] **Track-Aware Content Retrieval:** Update `CoreContentEngine` to support decimal track filtering (e.g., retrieving only Track 0 atoms).

---

## 🟨 Medium Priority: Advanced Logic & Ingestion
Refining the interaction between the "Muscle" and the "Spine".

### 3. Integrated Regex & Variant Support
* [ ] **Regex Flexibility:** Update the `ResolutionEngine` to support non-numeric indicators (e.g., "17:3a") and multi-chapter ranges natively.
* [ ] **Fractional Variant Alignment:** Implement logic in the `ContentEngine` to align multi-word translations (e.g., from `en.sahih.txt`) to single source atoms.

### 4. Semantic Search Infrastructure (Layer 4)
* [ ] **Vector Storage:** Implement `pgvector` columns in the `texts` table for semantic embeddings.
* [ ] **Hybrid Search Engine:** Upgrade `CoreSearchEngine` to return results based on a weighted combination of FTS and Vector similarity.

---

## 🟩 Low Priority: System Maturity
Standardizing the Gateway and improving API stability.

### 5. API & Error Standardization
* [ ] **Traversal Pagination:** Refactor `get_hierarchy` to return `Pagination<HierarchyNode>` to prevent data flooding on large book requests.
* [ ] **Domain-Specific Error Enum:** Replace `anyhow::Result` with a custom `ScriptureError` type mapping to specific HTTP status codes (404 for missing paths, 400 for bad coordinates).
* [ ] **Metadata Discovery Routes:** Implement `GET /api/v1/metadata` to allow clients to discover available WASM plugins (Spines) and translations.

---

## 🛠️ Module Development Checklist
* [ ] **Section 1: Data Layer**: UNIQUE constraint enforced on `(edition_id, absolute_index)`.
* [ ] **Section 2: Service Layer**: Every public engine function includes an `AI Prompt Hint`.
* [ ] **Section 3: Gateway Layer**: Routes use Axum 0.7+ `{variable}` syntax.



*This TODO list is governed by the vision of a "Stable Bottom, Liquid Top." Changes to the core sequence slots (The Muscle) must be minimized once seeded, while Spines (The Plugins) should remain highly iterative.*