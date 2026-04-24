# 📚 Scripture OS: Concept Documentation — Fractional Scripture Index (FSI)

## 0. The Metaphor

> **"The Big Scroll"**
> FSI treats an entire work (e.g., the Quran or Bible) not as a collection of isolated files, but as a single,
> immutable, and infinite linear sequence of "Atoms". Every word, letter, and variant occupies a precise, fractional
> position on this universal scroll.

---

## 1. The Concept (The "What")

**Definition:** Fractal Scripture Index (FSI) is a high-precision, multidimensional coordinate system designed for AI
ingestion and multi-tradition alignment. It replaces traditional book/chapter/verse numbering with a **3D Coordinate**
composed of a `WorkID`, `MacroID`, and a fractional `LexKey`.

**Domain Boundary:** This concept is the "Stable Bottom" of the OS, residing in `src/fsi/models.rs`. It is responsible
for the unique identity of scriptural data but is **not** responsible for the human-readable text (Lexicon) or the
navigational logic (Traversal Engine).

---

## 2. Architectural Intent (The "Why")

### ### Architectural Design Decision: Immutable Identity through Fractional Positioning

Traditional numbering systems fail when translations or variants differ in length or structure. FSI solves this by
allowing infinite horizontal insertions.

* **The "Stable Bottom":** By using fractional string keys (`LexKey`), we can insert a new word or variant between two
  existing atoms without re-indexing the entire database.
* **Decoupling:** FSI separates the **logical address** from the **linguistic content**. This allows multiple languages
  to occupy the same "Coordinate Stack," enabling instant side-by-side comparison.

---

## 3. Core Knowledge & Technical DNA

### The 3D Coordinate: `WorkID.MacroID.LexKey`

Every piece of data in Scripture OS is addressed by this three-part key.

| Component   | Type     | Example   | Responsibility                                     |
|:------------|:---------|:----------|:---------------------------------------------------|
| **WorkID**  | `i32`    | `786`     | Identifies the corpus (e.g., Quran).               |
| **MacroID** | `i32`    | `1`       | Identifies a major division (e.g., Surah/Chapter). |
| **LexKey**  | `String` | `00001.a` | The precise horizontal position (Word/Atom).       |

### Language & Namespace Taxonomy

FSI uses `NamespaceID` to separate data tracks within the same coordinate space.

| Namespace Range   | Language/Block    | Purpose                         |
|:------------------|:------------------|:--------------------------------|
| **1000 – 1999**   | Arabic (Original) | The "Skeleton" or Anchor track. |
| **10000 – 10999** | English           | Standard translation track.     |
| **20000 – 20999** | Spanish           | Additional linguistic tracks.   |

### Technical Context: Fractional Nesting

The `LexKey` is a string to allow for "infinite" depth. For example, if word `00001` has a variant, it is stored at
`00001.v1`. If a translation requires three words for one original word, they are indexed as `00001.a`, `00001.b`, and
`00001.c`.

---

## 4. Primary Use Cases

### Use Case A: Word-Level Multi-Language Alignment

When the API requests a verse, the engine pulls all atoms for a given `MacroID` and `LexKey` across different
`NamespaceIDs`.

* **Input:** `quran.1.1` (Alias)
* **Resolved Coordinate:** `786.1.00001`
* **Result:** * `786.1.00001` (Namespace 1000): بِسْمِ
    * `786.1.00001.a` (Namespace 10019): In
    * `786.1.00001.b` (Namespace 10019): the
    * `786.1.00001.c` (Namespace 10019): name

### Use Case B: Interleaved Scholarly Variants

If a manuscript has a different spelling for a word, it is inserted at a fractional position (e.g., `00001.v1`) without
breaking the sequential flow for other readers.

---

## 5. The "Versification Trap" (Common Problems)

* **Re-indexing Nightmare:** Traditional systems require shifting every subsequent ID if a new verse or word is
  discovered. FSI mitigates this via the **Integer Anchor** (MacroID) and **Decimal Insertion** (LexKey).
* **Boundary Leaks:** Developers often try to use `u32` for IDs. Scripture OS mandates `i32` and `i64` NewTypes to
  ensure seamless PostgreSQL integration and avoid unsigned-to-signed casting errors in the repository.
* **Ordering Logic:** Because `LexKey` is a string, simple numerical sorting fails (e.g., "10" comes before "2"). FSI
  mandates **lexicographical padding** (e.g., `00001`) to ensure correct database sorting.

---

## 6. AI Prompt Hints

> **Instruction for Future Agents:** When extending the FSI, always use the `Coordinate` struct from
`src/fsi/models.rs`. Never flatten these IDs into bare strings in the engine layer. LexKeys MUST be padded strings to
> maintain database sort integrity.

---

## 7. Dual-Track Verification Status

* **Track A (Integration):** Verified. `fsi_scroll` table correctly sorts and retrieves fractional LexKeys.
* **Track B (Mock):** Verified. `Coordinate` transformation logic passed all isolation tests.