This development plan leverages Rust's **Type System** and **Crate Ecosystem** to build a modular, high-performance
engine. By separating the "Dictionary" (Lexicon) from the "Map" (FSI Scroll), we ensure that the system is lean,
verifiable, and easy to extend.

---

## 🛠️ Module Architecture (DRY Principles)

To maximize code reuse and maintain strict boundaries, we will organize the project into the following internal modules.

### 1. `fsi_core` (The DNA)

* **Microservice Duty:** Shared Type Definitions.
* **Responsibility:** Defines the `Coordinate`, `LexKey`, `WorkID`, and `NamespaceID` types. It ensures that throughout
  the entire application, a "Work ID" can never be accidentally treated as a "Macro ID." It also houses the `Error`
  types shared across all modules.

### 2. `fsi_lexicon` (The Global Dictionary)

* **Microservice Duty:** Linguistic Content Management.
* **Responsibility:** Manages the `lexicon_entries` table. Its sole job is to perform "Get-or-Create" operations—taking
  a raw string and returning a `LexiconID`. It handles the storage of metadata like **Gematria values** and tri-literal
  roots.

### 3. `fsi_crypto` (The Notary)

* **Microservice Duty:** Cryptographic Integrity.
* **Responsibility:** Implements the **Merkle Tree** logic using the `blake3` crate. It takes a list of `AtomHashes` and
  rolls them up into `VerseRoots`, `ChapterRoots`, and the final `WorkRoot`. It provides the "Proof" that a piece of
  text hasn't changed since it was signed.

### 4. `fsi_storage` (The Vault)

* **Microservice Duty:** Database Persistence.
* **Responsibility:** Wraps `sqlx` to handle Postgres interactions. It isolates all SQL queries so that if we ever move
  from Postgres to a different engine (like ScyllaDB for massive scale), only this module needs to change.

### 5. `fsi_ingestor` (The Factory)

* **Microservice Duty:** Data Transformation (Write Path).
* **Responsibility:** Takes raw sources (Tanzil TXT, CSV, etc.) and converts them into FSI atoms. It coordinates with
  `fsi_lexicon` to get IDs and `fsi_crypto` to generate hashes before batch-saving them to `fsi_storage`.

### 6. `fsi_assembler` (The Weaver)

* **Microservice Duty:** Content Reconstitution (Read Path).
* **Responsibility:** Fetches a range of `LexKeys` across multiple `NamespaceIDs`. It "hydrates" the numerical stream by
  swapping `LexiconIDs` for actual text and groups them into verses/sentences for the end-user.

### 7. `fsi_virtual` (The Librarian)

* **Microservice Duty:** Virtual Scopes and Aliasing.
* **Responsibility:** Handles the **Pointer Logic** for custom collections (like your Prayer compilation). It tracks
  which virtual atoms point to which original source coordinates.

---

## 🚀 Phase-by-Phase Development Plan

### Phase 1: The Relational Foundation (The "Static" Bottom)

* **Action:** Update `init.sql` to separate `lexicon_entries` and `fsi_texts`.
* **Focus:** Implement `fsi_core` types and `fsi_lexicon` basic CRUD logic.
* **Deliverable:** A system that can store a word once and reference it by an ID.

### Phase 2: The Hash Pipeline (The "Seal")

* **Action:** Build `fsi_crypto`.
* **Focus:** Integrate `blake3` into the ingestion flow. Every time a word is saved, its hash is calculated and stored.
* **Deliverable:** A database where every row has a unique, verifiable cryptographic fingerprint.

### Phase 3: Multi-Language Interleaving (The "Loom")

* **Action:** Refactor `fsi_ingestor` and `fsi_assembler`.
* **Focus:** Implement the **Namespace Taxonomy** (1000 for Arabic, 10019 for Khalifa). Ensure the assembler can pull
  multiple tracks and align them by `LexKey`.
* **Deliverable:** A JSON API that returns side-by-side Arabic/English verses grouped word-by-word.

### Phase 4: Virtual Collections & Plugins (The "Lens")

* **Action:** Implement `fsi_virtual`.
* **Focus:** Create a "Collection" API where a user can select a range of words and save them as a new "Virtual Work"
  without duplicating text.
* **Deliverable:** The ability to generate a "Prayers" book that calls back to original Quranic verses.

---

## 📉 Efficiency Estimation: The "Most Efficient Path"

To reach MVP with maximum speed and minimal resource usage:

1. **Skip UI for now:** Build everything as **CLI Utilities** and **Integration Tests**. This ensures the data logic is
   100% sound before a single pixel is rendered.
2. **Use Batch Ingestion:** Instead of one SQL insert per word, use Postgres `COPY` or unnested arrays in `sqlx` to push
   10,000 words at a time. This reduces ingestion time from minutes to seconds.
3. **In-Memory Lexicon Cache:** During ingestion, keep a `HashMap<String, LexiconID>` in memory. This prevents the
   database from being hammered with "Does this word exist?" queries for common words like "the" or "Allah."

This plan ensures that every part of the system has one job (Microservice duty), never repeats logic (DRY), and stays
mathematically solid (Merkle/FSI).

Would you like to start by defining the **Rust Structs** for the new Lexicon-based `ScriptureAtom`?