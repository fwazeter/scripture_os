This development plan outlines the **Omni-Scripture Ledger (OSL)** as a modular, language-agnostic system. It treats
every scripture as a multi-layered data structure where the "Source" and "Translation" are synchronized tracks on a
shared timeline.

---

## Phase 1: The Data Architecture (Relational Ontology)

The database must be designed to separate **Sound/String** from **Meaning**.

### 1.1 The Concept Table (`registry_concepts`)

* **Purpose:** The language-neutral "ID" for an idea (e.g., "The Creator," "Mercy").
* **Fields:** `concept_id` (UUID), `primary_label` (String), `thesaurus_refs` (JSON).

### 1.2 The Lexicon Table (`lexicon_entries`)

* **Purpose:** Stores unique linguistic units.
* **Fields:**
    * `lexicon_id`: Primary Key.
    * `lang_code`: ISO 639-1 (AR, HE, SA, EN).
    * `raw_text`: The string.
    * `concept_id`: Link to Registry.
    * `morph_metadata`: JSON blob storing roots, stems, prefixes, or case markers.

### 1.3 The Scroll Table (`fsi_scroll`)

* **Purpose:** The "Spine." It uses **Hybrid Fractional Scripture Indexing** to place words in space.
* **Fields:**
    * `anchor`: Numerical (e.g., `100.00`, `110.00`).
    * `suffix`: Alpha (e.g., `a`, `b`) for alignment.
    * `variant_id`: ID identifying if this is "Arabic_Hafs" or "English_KJV."
    * `lexicon_id`: Foreign Key.
    * `node_hash`: BLAKE3 hash of this specific entry.

---

## Phase 2: Core Logic & Functional Modules

These functions should be implemented as a library (DLL, C++ Header, or Python Module) that can be called by any
interface.

### Module A: The Ingestion Engine

**Function:** `RegisterText(WorkID, Lang, VariantName, TextBlock)`

1. Tokenizes the text based on language rules (e.g., whitespace for English, Sandhi-splitting for Sanskrit).
2. Checks the `Lexicon` for existing strings. If missing, creates a new `lexicon_id`.
3. Assigns `anchors` at intervals of 10 to allow for future insertions.

### Module B: The Alignment Engine

**Function:** `AlignVariant(SourceAnchor, TargetLexiconID, Suffix)`

1. Takes a translation word and "docks" it to a Source Anchor.
2. If one Arabic word maps to three English words, it generates suffixes: `100a`, `100b`, `100c`.
3. Calculates the **Bridge Hash** (a hash of the Source word + its Translation word) to ensure they are forever linked.

### Module C: The Integrity Notary

**Function:** `SealVerse(WorkID, VerseNumber)`

1. Collects all `node_hashes` for a specific range of anchors.
2. Constructs a Merkle Tree where each leaf is a word/coordinate pair.
3. Returns the **Verse Root Hash**.

---

## Phase 3: Handling Linguistic Complexity

To ensure the system isn't "Arabic-centric" or "English-centric," the **Morphology Handler** must be modular:

* **For Semitic Languages (Arabic/Hebrew):** The logic uses the `morph_metadata` JSON to index by **3-Letter Roots**.
  Searching for a root returns all anchors associated with it.
* **For Indo-European (Greek/Sanskrit):** The logic indexes by **Stem and Case**.
* **For Agglutinative Languages (Pali/Sumerian):** The system treats the compound as a single Anchor, but uses the
  `Lexicon` to store sub-component pointers.

---

## Phase 4: Implementation Workflow (Example: Surah Fatiha)

1. **Step 1:** Ingest Arabic Verse 1. Database assigns Anchor `110` to `بِسْمِ`.
2. **Step 2:** Ingest English "Sahih International." The Alignment Engine sees "In" and "Name." It assigns them `110a`
   and `110b`.
3. **Step 3:** The Notary generates a hash for `110`, `110a`, and `110b`.
4. **Step 4:** If a user later adds a Hebrew variant, the system simply adds new rows at Anchor `110` with
   `variant_id = HE_TORAH`.
5. **Step 5:** **Verification.** A user runs `VerifyIntegrity()`. The system re-hashes every word. If a single
   translation word has been changed by a third party, the Merkle Root will fail to match the "Work Root Hash."

---

## Summary of System Capabilities

| Capability               | Technical Solution                             |
|:-------------------------|:-----------------------------------------------|
| **Language Agnostic**    | JSON Metadata + ISO Lang Tags.                 |
| **Translation Equality** | Treated as "Variants" using the Suffix System. |
| **Zero Redundancy**      | Global Lexicon de-duplication.                 |
| **Total Integrity**      | Multi-tiered Merkle Trees.                     |
| **Fast Comparison**      | Direct Anchor-to-Anchor JOINS.                 |

This plan moves scripture from "digital paper" to a **verifiable linguistic ledger**.

Would you like to explore the specific SQL schema for the `morph_metadata` JSON to see how it would store Hebrew
prefixes vs. Sanskrit compounds?