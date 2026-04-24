# 📚 Scripture OS: Concept Documentation Template

## 0. The Metaphor

> Provide a high-level mental model (e.g., "The Spine," "The Guide," or "The DNA") to establish immediate context.

---

## 1. The Concept (The "What")

**Definition:** A concise technical definition of the feature or module.
**Domain Boundary:** State where this lives (e.g., `src/fsi`, `src/engines`) and what it is **not** responsible for to
ensure strict separation of concerns.

---

## 2. Architectural Intent (The "Why")

### ### Architectural Design Decision: [Title of Core Philosophy]

Explain the specific problem this concept solves.

* **The "Stable Bottom":** How does this contribute to the immutable foundation of the system?
* **Decoupling:** What dependencies are being avoided or abstracted? (e.g., Trait-based DI or Anti-Corruption Layers).

---

## 3. Core Knowledge & Technical DNA

This section defines the "Atomic" requirements.

| Component       | Type/Contract | Responsibility                                |
|:----------------|:--------------|:----------------------------------------------|
| **Identifier**  | `NewType`     | Ensures compile-time safety (e.g., `WorkID`). |
| **Logic Layer** | `Trait`       | The contract that must be fulfilled.          |
| **State**       | `Struct`      | The data held by the engine or model.         |

### Technical Context: [Specific Logic]

Explain non-obvious details like `blake3` hashing, lexicographical sorting, or fractional indexing (LexKey).

---

## 4. Primary Use Cases

* **Use Case A:** Describe a primary flow (e.g., "Resolving a human alias like 'quran.1.1'").
* **Use Case B:** Describe a background flow (e.g., "Batch-ingesting 6,000+ verses via the CLI").

---

## 5. The "Versification Trap" (Common Problems)

Identify the pitfalls and how the architecture mitigates them:

* **Data Integrity:** How we prevent duplicate entries using `ON CONFLICT` logic.
* **Formatting Quirks:** Lessons learned from string escaping or terminal rendering (e.g., the `\` character in SQL
  strings).
* **Boundary Leaks:** Warning against letting implementation details (like Postgres/SQLx) leak into the domain models.

---

## 6. AI Prompt Hints

> **Instruction for Future Agents:** Provide specific rules for AI when modifying this concept. (e.g., "Never change the
`merkle_hash` byte array size without updating the crypto validation logic").

---

## 7. Dual-Track Verification Status

* **Track A (Integration):** Status of PostgreSQL tests.
* **Track B (Mock):** Status of isolated logic tests.

---

### How to use this template:

We can now apply this to our existing features. Which one should we document first: the **Fractional Scripture Index (
FSI)**, the **Strategy-Based Ingestion Pipeline**, or the **Versioned API Gateway**?