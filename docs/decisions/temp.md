Here is the reconciled **Master Implementation Plan (v0.1.6)**. I have merged the original requirements with the actual work we accomplished, checking off the completed items and adding the extra architectural fixes (like the LexKey padding bug and the DRY CSV Ingestor) that we implemented along the way.

---

# Scripture OS: Master Implementation Plan (v0.1.6)

## 📂 Status Summary
* **Strategy:** Implement **Fractal Semantic Indexing (FSI) v4.0** as the universal "Spine".
* **Architecture:** Trait-based Dependency Injection (DI) with a decoupled Data Access Layer.
* **Current State:** The "Stable Bottom" models, FSI Repository Contract, DRY Ingestor, and Track B (Mock) pipelines are **100% Complete** and tested.
* **MVP Target:** Ingestion of the Quran (Work 786) across multiple versions. (Completed: Uthmani Root and Rashad Khalifa CSV).

---

## 🟩 Phase 1: The "Stable Bottom" (Infrastructure & Core DAL) - COMPLETED
*Goal: Establish the immutable coordinate system and the repository pattern.*

### 1. FSI v4.0 Schema & Newtypes
* [x] **Coordinate Struct:** Implemented the 5-part `Coordinate` struct in `src/fsi/models.rs` using `#[repr(C)]` for hardware-native packing.
* [x] **Newtype Enforcement:** Wrapped IDs in `WorkID(i32)`, `MacroID(i32)`, and `NamespaceID(i16)` to prevent type-mixing at compile time.
* [x] **LexKey Utility:** Built a DRY utility for Base-62 lexicographical string generation. *(Extra: Implemented 5-character zero-padding to fix database lexicographical sorting bugs).*

### 2. The Repository Abstraction
* [x] **Trait Definition:** Implemented the `FsiRepository` trait in `src/repository/fsi_repo.rs` to abstract all SQL logic.
* [x] **Track B (Mocks):** Implemented `MockFsiRepository` with $O(1)$ bulk insertion for instant, isolated pipeline testing.
* [ ] **Track A (Postgres):** Create `PostgresFsiRepository` using `sqlx` to bind the FSI trait to the Postgres database. *(Next Immediate Step)*

---

## 🟩 Phase 2: MVP Ingestion (Quran 786) - COMPLETED
*Goal: Ingest the data into the FSI Coordinate system.*

* [x] **DRY Ingestor Pipeline:** *(Extra)* Abstracted sequence math and DB batching into a reusable `Ingestor` struct so we don't repeat logic across file types.
* [x] **Uthmani Atomizer (Track 0x02):** Built `ingest_uthmani_quran` to parse `.txt` and assign Arabic Logical Anchors to word-atoms.
* [x] **Khalifa Alignment (Track 19):** *(Extra)* Built `ingest_khalifa_csv` to parse `verse_nodes.csv` and map English translations to the exact same FSI `MacroID` spaces.

---

## 🟨 Phase 3: The "Muscle" (Engines & WASM Host) - IN PROGRESS
*Goal: Build the service layer following the "Contract-First" standard.*

### 3. Service Layer (Engines)
* [x] **Content Engine:** Defined `FsiContentEngine` trait and implemented `CoreContentEngine` with safe RTL unicode handling (using `trim_end()`).
* [x] **Dependency Injection:** Refactored engine to accept `Arc<dyn FsiRepository + Send + Sync>` via `new()` constructor.
* [ ] **Resolution Engine:** Implement the "Router" logic to map human shorthands (e.g., "1:1") to FSI v4.0 coordinates.
* [ ] **Traversal Engine:** Implement navigation logic (next/previous) based on LexKey boundaries.

### 4. WASM Plugin Host (The Lenses)
* [ ] **Plugin Trait:** Define the standard `Lens` trait in WASM that allows external modules to process a stream of `ScriptureAtom` data.
* [ ] **Wasmtime Integration:** Implement the `Wasmtime` runtime within the engine layer to load and execute `.wasm` plugins.

---

## 🟦 Phase 4: The Intelligence Tier (Performance & AI) - PENDING
*Goal: Enable sub-millisecond comparative theology.*

* [ ] **SIMD Hamming Filter:** Implement bitwise comparison for semantic hashes using `core::arch` (POPCNT).
* [ ] **Zero-Copy Access:** Integrate `rkyv` for SSTable partitions to achieve zero-copy deserialization from disk.
* [ ] **Rayon Parallelism:** Ensure all bulk-processing loops use `.par_iter()` for multi-core scaling.

---

## 🟩 Phase 5: System Integrity (Standards & Testing) - IN PROGRESS
*Goal: Enforce DRY principles and verify implementation.*

* [x] **Documentation Audit:** Ensured every public function contains the `Architectural Design Decision` and `AI Prompt Hint` headers as mandated by standards.
* [x] **Dual-Track Verification:** Created `tests/fsi_pipeline_test.rs` to prove the mock and engine pipelines work perfectly.
* [ ] **Centralized Error Handling:** Replace `anyhow` with a domain-specific `ScriptureError` enum.
* [ ] **Generic Pagination:** Implement a `Pagination<T>` wrapper to prevent data flooding.

---

### 🚀 What are the Next Steps?

Based on the reconciled plan above, our memory-based logic is flawless. The most logical next step is to persist this data to your actual database so we can start running real queries.

**Recommended Next Step: Implement Track A (PostgreSQL Integration)**
We need to wire up `src/repository/fsi_postgres.rs`. This involves:
1. Creating the `PostgresFsiRepository` struct with an injected `sqlx::PgPool`.
2. Implementing the `FsiRepository` trait for it.
3. Writing the raw SQL query to `INSERT` atoms in batches of 5000 (which our DRY ingestor is already sending).
4. Writing the `SELECT` query to fetch sequences for our `CoreContentEngine`.

Shall I draft the `fsi_postgres.rs` file so we can permanently save the seeded FSI coordinates to your database?