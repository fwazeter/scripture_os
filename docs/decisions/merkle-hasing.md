To show you what the **Bismillah** (Surah Al-Fatihah 1:1) looks like in our new architecture, we must view it across two
distinct tables: the **Universal Lexicon** (the dictionary of unique words) and the **FSI Coordinate Scroll** (the
interleaved stream of addresses).

### 1. The Universal Lexicon Table (`lexicon_entries`)

This table stores every unique linguistic "atom" once. For the Bismillah, it looks like this:

| Lexicon ID | Lang Block      | Raw Atom  | Morphology (JSONB)                            |
|:-----------|:----------------|:----------|:----------------------------------------------|
| `1001`     | 1000 (Arabic)   | `بِسْمِ`  | `{"root": "bsm", "type": "preposition+noun"}` |
| `1002`     | 1000 (Arabic)   | `ٱللَّهِ` | `{"root": "alh", "type": "proper_noun"}`      |
| `5001`     | 10000 (English) | `In`      | `{"type": "preposition"}`                     |
| `5002`     | 10000 (English) | `the`     | `{"type": "article"}`                         |
| `5003`     | 10000 (English) | `name`    | `{"type": "noun"}`                            |
| `5004`     | 10000 (English) | `of GOD`  | `{"type": "phrase", "semantic_link": 1002}`   |

---

### 2. The FSI Coordinate Scroll (`fsi_texts`)

This is the "interleaved" stream where the actual scripture lives. By sorting by `LexKey` and then `NamespaceID`, we see
the Arabic root and the English translations grouped word-by-word.

| Work      | Macro   | LexKey      | Namespace | Lexicon ID | SubMask | Merkle Hash (Blake3) |
|:----------|:--------|:------------|:----------|:-----------|:--------|:---------------------|
| `786`     | `1`     | `00000`     | 1000      | `0`        | `4`     | `H_MARK_A...`        |
| `786`     | `1`     | `00000`     | 10019     | `0`        | `4`     | `H_MARK_B...`        |
| **`786`** | **`1`** | **`00001`** | **1000**  | **`1001`** | **`1`** | **`H_BSM...`**       |
| `786`     | `1`     | `00001.a`   | 10019     | `5001`     | `0`     | `H_IN...`            |
| `786`     | `1`     | `00001.b`   | 10019     | `5002`     | `0`     | `H_THE...`           |
| `786`     | `1`     | `00001.c`   | 10019     | `5003`     | `0`     | `H_NAME...`          |
| **`786`** | **`1`** | **`00002`** | **1000**  | **`1002`** | **`1`** | **`H_ALLAH...`**     |
| `786`     | `1`     | `00002`     | 10019     | `5004`     | `0`     | `H_OFGOD...`         |

---

### Key Takeaways from this Entry:

* **Interleaved Logic**: Notice how `00001` (Arabic: `بِسْمِ`) is immediately followed by its English fractional keys
  `00001.a`, `b`, and `c`. In a side-by-side comparison view, the engine simply pulls this block to show exactly how the
  Arabic word maps to the English phrase.
* **Cryptographic Integrity**: The **Merkle Hash** for `00001` (Arabic) is calculated by hashing the `Lexicon Content` +
  the `Coordinate`. If a single character is changed in the lexicon or the address, the hash breaks.
* **The SubMask Filter**: The Arabic words have a `SubMask` of `1` (**Logical Anchor**), while the English words have
  `0`. This tells the system that the Arabic text is the "Skeleton" and the English text is the "Translation Layer".
* **Namespace Taxonomy**: We see **1000** (Arabic Block) and **10019** (English Block: Rashad Khalifa) being used,
  keeping the database human-readable.

This structure allows us to scale down to the **letter** (using further fractional keys like `00001.a.1`) or up to the *
*Work** (by hashing all verse roots into a single Work Hash).

Shall we implement the **Merkle-Hashing logic** in our `Ingestor` so it generates these fingerprints automatically as it
processes the new language blocks?