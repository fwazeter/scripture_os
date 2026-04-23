# Scripture OS: Namespace ID Taxonomy (FSI v4.0)

## Overview

In Fractal Semantic Indexing (FSI) v4.0, the `NamespaceID` represents the specific track, language, or edition of a
`WorkID`.

To maintain a human-readable database and ensure global scalability across all biblical and scriptural languages, we
utilize a **Base-10 Block Architecture** within the bounds of a 16-bit signed integer (`i16`), which has a maximum value
of `32,767`.

## The Block Pattern

The `NamespaceID` is derived using the following pattern:
**`[Language Block Prefix] + [Edition Identifier]`**

Each language is assigned a prefix block of 1,000 slots. The last three digits (`000` to `999`) identify the specific
manuscript or translation.

### 1. System & Original Root Languages

Root scripts and ancient languages occupy the lower integer blocks (`0` to `9999`).

| Block Range   | Language / Category   | Example Editions                                   |
|:--------------|:----------------------|:---------------------------------------------------|
| `0 - 999`     | **System & Metadata** | `0` (Logical Anchor), `16` (AI PQ-Hashes)          |
| `1000 - 1999` | **Arabic Texts**      | `1000` (Uthmani Root), `1001` (Simple Clean)       |
| `2000 - 2999` | **Hebrew / Aramaic**  | `2000` (Masoretic Text), `2001` (Dead Sea Scrolls) |
| `3000 - 3999` | **Greek Texts**       | `3000` (Textus Receptus), `3001` (Septuagint)      |
| `4000 - 4999` | **Sanskrit / Pali**   | `4000` (Vedic Sanskrit Root)                       |

### 2. Modern Translation Languages

Translated tracks occupy the upper blocks (`10000` to `32000`), leaving massive room for expansion.

| Block Range     | Language    | Specific Edition Examples                                                                              |
|:----------------|:------------|:-------------------------------------------------------------------------------------------------------|
| `10000 - 10999` | **English** | `10019` (Rashad Khalifa)<br>`10020` (Sahih International)<br>`10021` (Yusuf Ali)<br>`10161` (KJV 1611) |
| `11000 - 11999` | **Spanish** | `11000` (Reina-Valera)                                                                                 |
| `12000 - 12999` | **French**  | `12000` (Louis Segond)                                                                                 |
| `13000 - 13999` | **Urdu**    | `13000` (Jalandhari)                                                                                   |
| `...`           | *Reserved*  | Up to limit `32,767`                                                                                   |

## Rules for Assignment

1. **Never Exceed 32,767:** The database column is a `smallint` for extreme query performance. Any ID above `32,767`
   will crash the database layer.
2. **000 is the Root:** The `000` edition in any language block (e.g., `1000`, `2000`) should generally be reserved for
   the most widely accepted "Root" version of that language.
3. **Intentional Overrides:** Specific edition identifiers can be manually chosen for theological or mnemonic reasons (
   e.g., Rashad Khalifa is explicitly placed at `10019` due to the mathematical significance of 19 in his work).