# 📚 Scripture OS: Concept Documentation — The Scroll Notary (Hashing System)

## 0. The Metaphor

> **"The Notary"**
> If the FSI is the address and the Lexicon is the content, the Hashing System is the **Notary** that stamps every
> single atom with an immutable fingerprint. It provides a cryptographic guarantee that not a single dot or tittle has
> been altered, across any language or translation track.

---

## 1. The Concept (The "What")

**Definition:** The Scroll Notary is a multi-layered cryptographic integrity system built on **Merkle Trees** and the *
*BLAKE3** hashing algorithm. It rolls up individual "Atom Hashes" into a hierarchy of "Roots" (Verse, Chapter, and
Work), allowing for instant verification of massive datasets with a single 32-byte string.

**Domain Boundary:** This system resides in the `fsi_crypto` module and is utilized primarily by the **Ingestion Engine
** (to generate fingerprints) and the **Verification Engines** (to audit state). It is **not** responsible for storing
the text itself, but rather for signing the *state* of the text at a specific FSI coordinate.

---

## 2. Architectural Intent (The "Why")

### ### Architectural Design Decision: Cross-Track Merkle Convergence

Traditional hashing treats different translations as separate files. Scripture OS uses **Cross-Track Convergence** to
hash all versions of a verse into a single "Universal State".

* **The "Stable Bottom":** By binding the `LexiconID` and the `FSI Coordinate` into the hash, we ensure that an atom is
  only valid if it is exactly the right word in exactly the right place.
* **Decoupling:** This allows for **Zero-Knowledge Comparison**. Two servers can verify they have the exact same Quranic
  text in 50 languages by comparing one 32-byte **Work Root Hash** without ever exchanging a single word of text.

---

## 3. Core Knowledge & Technical DNA

### The Vertical Roll-up (The Hierarchy)

The system converges data from the bottom up:

| Level         | Component               | Calculation                                         |
|:--------------|:------------------------|:----------------------------------------------------|
| **L1: Atom**  | `Atom Hash`             | `blake3(LexiconID + FSI Coordinate)`                |
| **L2: Stack** | `Coordinate Stack Hash` | `blake3(Sum of all Namespace hashes at one LexKey)` |
| **L3: Verse** | `Verse Root`            | `blake3(Ordered list of Stack Hashes in a Verse)`   |
| **L4: Macro** | `Chapter Root`          | `blake3(Ordered list of Verse Roots)`               |
| **L5: Work**  | `Work Root`             | `blake3(All Chapter Roots)`                         |

### Technical Context: BLAKE3 Performance

We use **BLAKE3** because it is a "Bao-ready" tree-based hash that is significantly faster than SHA-256 or SHA-3, which
is critical when hashing 77,000+ words of a single work in real-time during ingestion.

---

## 4. Primary Use Cases

### Use Case A: Instance Synchronization & Auditing

Two independent nodes running Scripture OS can perform a "handshake" by exchanging their **Work Root Hash**. If the
hashes match, both nodes are mathematically guaranteed to be in a perfectly synchronized state across all languages.

### Use Case B: Tamper Detection

If a single character is changed in a specific English translation (Namespace 10019), it changes that specific **English
Atom Hash**. This change "bubbles up" through the **Coordinate Stack Hash** and **Verse Root**, eventually altering the
**Work Root Hash**, signaling an immediate integrity failure for that work.

---

## 5. The "Versification Trap" (Common Problems)

* **The Sensitivity Pitfall:** Because the hash includes the `Coordinate`, moving a word from `00001` to `00001.a` will
  break the hash even if the text is identical. The hash proves **Placement**, not just **Content**.
* **Ordering Logic:** Merkle trees are order-dependent. If the database returns verses out of order during a roll-up,
  the resulting hash will be incorrect. FSI mandates **strict lexicographical sorting** by `LexKey` during all hashing
  operations.
* **Namespace Drift:** If one server has 3 languages and another has 4, their **Stack Hashes** will never match. The
  system must specify the "Namespace Scope" when performing an audit.

---

## 6. AI Prompt Hints

> **Instruction for Future Agents:** When implementing the `fsi_crypto` module, ensure the hashing input always follows
> the standard sequence: `[WorkID][MacroID][LexKey][NamespaceID][LexiconID]`. Never allow salt or random seeds in these
> hashes; they must be **deterministic** across all OS instances.

---

## 7. Dual-Track Verification Status

* **Track A (Integration):** Verified. `merkle_hash` column in `fsi_scroll` correctly stores 32-byte BLAKE3 outputs.
* **Track B (Mock):** Planned. Logic for rolling up Chapter Roots from Verse Roots is currently being simulated in unit
  tests.