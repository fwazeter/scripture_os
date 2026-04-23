This final implementation plan for **Scripture OS** synthesizes the architectural refinements of **Plan V2** with the performance-first primitives of **Plan V1**. It adheres to the **"Data-as-Core, Logic-as-Plugin"** philosophy to ensure the system is modular, DRY, and re-usable.

---

# Scripture OS: Master Implementation Plan (v0.1.5)

## 📂 Status Summary
* **Strategy:** Implement **Fractal Semantic Indexing (FSI) v4.0** as the universal "Spine".
* **Architecture:** Trait-based Dependency Injection (DI) with a decoupled Data Access Layer.
* **MVP Goal:** Full ingestion of the Quran (Work 786) across three versions (Uthmani, Sahih, Pickthall) using word-level atoms.

---

## 🟥 Phase 1: The "Stable Bottom" (Infrastructure & Core DAL)
*Goal: Establish the immutable coordinate system and the repository pattern.*

### 1. FSI v4.0 Schema & Newtypes
* [x] **Coordinate Struct:** Implement the 5-part `Coordinate` struct in `src/models.rs` using `#[repr(C)]` for hardware-native packing.
* [x] **Newtype Enforcement:** Wrap IDs in `WorkID(i32)`, `MacroID(i32)`, and `NamespaceID(i16)` to prevent type-mixing at compile time.
* [x] **LexKey Utility:** Build a DRY utility for Base-62 lexicographical string generation to allow infinite word-level insertion.

### 2. The Repository Abstraction
* [x] **Trait Definition:** Implement the `ScriptureRepository` trait in `src/repositories/mod.rs` to abstract all SQL logic.
* [x] **Track B (Mocks):** Implemented `MockFsiRepository` with O(1) bulk insertion for instant, isolated pipeline testing.
* [ ] **Track A (Postgres):** Create `PostgresFsiRepository` using `sqlx` to bind the FSI trait to the Postgres database. *(Next Immediate Step)*
* [x] **DRY Ingestor Pipeline:** *(Extra)* Abstracted sequence math and DB batching into a reusable `Ingestor` struct so we don't repeat logic across file types.
* [x] **Uthmani Atomizer (Track 0x02):** Built `ingest_uthmani_quran` to parse `.txt` and assign Arabic Logical Anchors to word-atoms.
* [x] **Khalifa Alignment (Track 19):** *(Extra)* Built `ingest_khalifa_csv` to parse `verse_nodes.csv` and map English translations to the exact same FSI `MacroID` spaces.
* [ ] **Postgres Implementation:** Create `PostgresRepository` using `sqlx`. Ensure all queries use `ltree` for hierarchical pathing.
* [ ] **DRY Query Builders:** Create re-usable SQL fragment builders for common operations like "fetch by coordinate range" to avoid duplicating complex JOIN logic.
* [x] **Documentation Audit:** Ensured every public function contains the `Architectural Design Decision` and `AI Prompt Hint` headers as mandated by standards.
* [x] **Dual-Track Verification:** Created `tests/fsi_pipeline_test.rs` to prove the mock and engine pipelines work perfectly.
---

## 🟨 Phase 2: The "Muscle" (Engines & WASM Host)
*Goal: Build the service layer following the "Contract-First" standard.*

### 3. Service Layer (Engines)
* [ ] **Contract-First Traits:** Define `ResolutionEngine`, `ContentEngine`, and `TraversalEngine` traits before implementation.
* [ ] **Dependency Injection:** Refactor engines to accept `Arc<dyn ScriptureRepository + Send + Sync>` via their `new()` constructors.
* [x] **Content Engine:** Defined `FsiContentEngine` trait and implemented `CoreContentEngine` with safe RTL unicode handling (using `trim_end()`).
* [x] **Dependency Injection:** Refactored engine to accept `Arc<dyn FsiRepository + Send + Sync>` via `new()` constructor.
* [ ] **Resolution Engine:** Implement the "Router" logic to map human shorthands to FSI v4.0 coordinates.

### 4. WASM Plugin Host (The Lenses)
* [ ] **Plugin Trait:** Define the standard `Lens` trait in WASM that allows external modules to process a stream of `ScriptureContent`.
* [ ] **Wasmtime Integration:** Implement the `Wasmtime` runtime within the engine layer to load and execute `.wasm` plugins (e.g., a "Tajweed Lens" for Quranic analysis).

---

## 🟦 Phase 3: The Intelligence Tier (Performance & AI)
*Goal: Enable sub-millisecond comparative theology.*

### 5. Hardware-Accelerated Logic
* [ ] **SIMD Hamming Filter:** Implement bitwise comparison for semantic hashes using `core::arch` (POPCNT).
* [ ] **Zero-Copy Access:** Integrate `rkyv` for SSTable partitions to achieve zero-copy deserialization from disk.
* [ ] **Rayon Parallelism:** Ensure all bulk-processing loops use `.par_iter()` for multi-core scaling.

---

## 🟩 Phase 4: System Integrity (Standards & Testing)
*Goal: Enforce DRY principles and verify implementation.*

### 6. DRY & Re-usability Steps
* [ ] **Centralized Error Handling:** Replace `anyhow` with a domain-specific `ScriptureError` enum that provides consistent error codes across all engines.
* [ ] **Generic Pagination:** Implement a `Pagination<T>` wrapper used by all traversal and search routes to prevent data flooding.
* [ ] **Documentation Audit:** Ensure every public function contains the `Architectural Design Decision` and `AI Prompt Hint` headers as mandated by standards.

### 7. Dual-Track Verification
* [ ] **Track A (Integration):** Implement `tests/db_integration_test.rs` using `test_utils::setup_db()` for real-world validation.
* [ ] **Track B (Mocks):** Implement `MockRepository` for every engine to test business logic (like coordinate math) in total isolation.

---

## 🛠️ Implementation Checklist for Every New Component
1.  **Define the Trait:** Does a contract exist in `src/engines/mod.rs`?
2.  **Apply Newtypes:** Are we passing raw integers where we should use `WorkID`?
3.  **Inject Dependencies:** Are database pools hardcoded, or injected via `Arc<dyn Trait>`?
4.  **Add AI Hints:** Is there an `AI Prompt Hint` in the docstring to guide future model-based edits?
5.  **Check Memory Layout:** Is the struct using `SmallVec` for LexKeys to minimize heap allocations?