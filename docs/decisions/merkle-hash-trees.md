To visualize the MVP for Surah Al-Fatihah, we must look at the **"Horizontal Interleaving"** of the database tables and
the **"Vertical Roll-up"** of the cryptographic hashes. This structure transforms the 7 verses into a single, verifiable
Chapter Hash.

### 1. The Merkle Convergence (The Hash Tree)

All 7 verses of Al-Fatihah are hashed individually to create **Verse Roots**. These roots are then hashed together to
form the **Chapter Root Hash**.

* **Chapter Root (Al-Fatihah):** `0xAF12...` (A single 32-byte hash representing the entire chapter).
    * **Verse 1 Root:** `0xV1...` (Calculated from all word-atoms in Verse 1).
    * **Verse 2 Root:** `0xV2...` (Calculated from all word-atoms in Verse 2).
    * *...Repeating for all 7 verses...*

---

### 2. The Table Interplay (MVP Blueprint)

The system "unpacks" data by using the **FSI Scroll** as a sequence of pointers that "hydrate" themselves from the *
*Universal Lexicon**.

#### Table A: `lexicon_entries` (The Language Vault)

This table stores the "what"—the unique words across all languages.

* **ID 1001**: `بِسْمِ` (Arabic Root Block: 1000).
* **ID 5001**: `In` (English Translation Block: 10000).
* **ID 5002**: `the`.
* **ID 5003**: `name`.

#### Table B: `fsi_texts` (The Coordinate Scroll)

This table stores the "where" and the "integrity"—the addresses and hashes.

* **Row 1**: `786.1.00001` | Namespace `1000` | LexiconID `1001` | Hash `0xA1...`.
* **Row 2**: `786.1.00001.a` | Namespace `10019` | LexiconID `5001` | Hash `0xB2...`.

---

### 3. Unpacking Surah Al-Fatihah: A Data Snapshot

This is how the first verse (Bismillah) is reconstructed between the tables.

| Coordinate      | Namespace (Lang) | Lexicon ID | SubMask | Atom Hash | Unpacked Text |
|:----------------|:-----------------|:-----------|:--------|:----------|:--------------|
| `786.1.00000`   | 1000 (Arabic)    | `0`        | `4`     | `H_MARK`  | `VERSE:1`     |
| `786.1.00001`   | 1000 (Arabic)    | `1001`     | `1`     | `H_BSM`   | `بِسْمِ`      |
| `786.1.00001.a` | 10019 (English)  | `5001`     | `0`     | `H_IN`    | `In`          |
| `786.1.00001.b` | 10019 (English)  | `5002`     | `0`     | `H_THE`   | `the`         |
| `786.1.00001.c` | 10019 (English)  | `5003`     | `0`     | `H_NAME`  | `name`        |
| `786.1.00002`   | 1000 (Arabic)    | `1002`     | `1`     | `H_ALLAH` | `ٱللَّهِ`     |

---

### 4. How it All Connects (The Flow)

1. **Request:** The user asks for "Al-Fatihah" (Work 786, Macro 1).
2. **Fetch:** The `ContentEngine` queries `fsi_texts` for all LexKeys in that Macro range.
3. **Integrity Check:** The engine verifies the `Atom Hashes` roll up into the `Chapter Root Hash` stored in the system
   metadata.
4. **Hydration:** The engine swaps the `LexiconIDs` with the `raw_atom` text from the Lexicon table.
5. **Assembly:** The Engine uses the `VERSE:X` markers (SubMask 4) to group words into sentences for the final UI.

This architecture provides **size savings** by storing integers instead of strings and **absolute security** by ensuring
the Chapter Hash only stays valid if every single word and coordinate remains exactly as it was during ingestion.