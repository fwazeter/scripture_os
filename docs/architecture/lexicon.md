# 📚 Scripture OS: Concept Documentation — The Lexicon

## 0. The Metaphor

> **"The Flesh"**
> If the FSI Scroll is the skeletal structure providing the "where," the Lexicon is the flesh providing the "what". It
> is the universal dictionary where every unique string of scriptural text is stored exactly once.

---

## 1. The Concept (The "What")

**Definition:** The Lexicon is a deduplicated, relational storage system for all linguistic content within Scripture OS.
It separates the raw text from its structural address, allowing the same string of text to be referenced by multiple
coordinates or versions.

**Domain Boundary:** The Lexicon spans the `src/repository` layer (for persistence) and the `src/engines/content.rs` (
for resolution). It is **not** responsible for determining the order of verses or handling human-readable aliases; its
sole duty is the high-performance storage and retrieval of strings based on a `LexiconID`.

---

## 2. Architectural Intent (The "Why")

### ### Architectural Design Decision: Content De-duplication & Normalization

Scripture often contains repetitive phrases or identical verses across different manuscripts or translations. By storing
text in a centralized Lexicon:

* **The "Stable Bottom":** We ensure that the core content is stored in a normalized form, reducing database bloat and
  making the system "AI-ready" for cross-linguistic comparison.
* **Decoupling:** The structural "address" of a verse (FSI Coordinate) is completely decoupled from its linguistic "
  value". This allows us to change the versification of a work without ever touching or duplicating the actual text.

---

## 3. Core Knowledge & Technical DNA

The Lexicon relies on a strict ID-to-Text mapping.

| Component       | Type/Contract          | Responsibility                                                                                                                         |
|:----------------|:-----------------------|:---------------------------------------------------------------------------------------------------------------------------------------|
| **Identifier**  | `LexiconID`            | A NewType wrapping an `i64` (Postgres `BIGINT`) to ensure we never accidentally pass a WorkID where a dictionary pointer is expected.  |
| **Persistence** | `fsi_lexicon` Table    | A PostgreSQL table with a `UNIQUE` constraint on the `body_text` column to enforce de-duplication at the hardware level.               |
| **Logic Layer** | `insert_lexicon_entry` | An idempotent repository method that either inserts new text or returns the ID of existing text using a CTE (Common Table Expression). |

### Technical Context: Idempotent Insertion

The Lexicon uses a specialized SQL pattern: `INSERT ... ON CONFLICT (body_text) DO NOTHING`. This allows the Ingestion
Engine to blindly submit text while the Repository ensures that the dictionary only grows when truly unique content is
introduced.

---

## 4. Primary Use Cases

* **Use Case A (Ingestion):** During the execution of the `seed` binary, the `CoreIngestionEngine` passes raw strings to
  the repository. The repository handles the Lexicon logic, returning a `LexiconID` which is then bound to a
  `ScriptureAtom`.
* **Use Case B (Assembly):** When a user requests a verse via the API, the `CoreContentEngine` first retrieves the
  `ScriptureAtom` to get the `lexicon_id`, then calls `get_lexicon_text` to resolve that ID into the final Arabic or
  English string for the user.

---

## 5. The "Versification Trap" (Common Problems)

* **Data Integrity:** Without the `UNIQUE` constraint on the `body_text` column, the Lexicon would quickly fill with
  thousands of duplicate "بِسْمِ ٱللَّهِ..." entries, breaking the "Universal Dictionary" promise.
* **Boundary Leaks:** It is tempting to store the text directly in the `fsi_scroll` table for speed. However, this
  violates the architecture by mixing "addressing" with "content," making cross-translation comparison significantly
  harder for AI models.
* **Relational Integrity:** Every `lexicon_id` in the `fsi_scroll` must correspond to a valid entry in `fsi_lexicon`.
  This is enforced via a foreign key constraint in the `init.sql` schema.

---

## 6. AI Prompt Hints

> **Instruction for Future Agents:** When implementing new engines, never allow the engine to query `fsi_lexicon`
> directly. Always use the `ScriptureRepository` trait methods `insert_lexicon_entry` or `get_lexicon_text` to maintain
> the Anti-Corruption Layer.

---

## 7. Dual-Track Verification Status

* **Track A (Integration):** Verified. `insert_lexicon_entry` correctly handles de-duplication in PostgreSQL.
* **Track B (Mock):** Verified. `MockRepository` successfully simulates Lexicon resolution for high-speed logic testing.