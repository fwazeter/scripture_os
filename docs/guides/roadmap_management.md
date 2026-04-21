
# Task Management Standards

This guide establishes the protocol for creating and maintaining the `docs/decisions/todo.md` to ensure maximum clarity for both human developers and AI collaborators.

#### **A. The Philosophy of "Contextual Tasks"**
A TODO item in Scripture OS is not just a reminder; it is a mini-blueprint. Every task must prioritize the **"Why"** (Architectural Intent) and link back to the **Spine and Muscle** architecture.

#### **B. Task Anatomy**
Efficient tasks must follow this structure:
* **Actionable Title:** Use imperative verbs (e.g., "Implement...", "Refactor...").
* **Specific Sub-tasks:** Break down the work into Trait, Repository, Engine, and Gateway changes.
* **Citations:** Every task should cite the relevant architectural document or source file it impacts.
* **Testing Requirement:** Explicitly mention the **Dual-Track Verification** (Mock and Integration).

#### **C. Maintenance Workflow**
1.  **Consolidate:** When a phase is reached, extract remaining items from archived plans into the active `todo.md`.
2.  **Verify:** Check off items only after all tests (Track A and Track B) pass green.
3.  **Archive:** Once a section is 100% complete, move the high-level summary to `docs/decisions/archive/`.

---

### Module Development Checklist
**Proposed Path:** `docs/guides/development_checklists.md`

Use these checklists when creating or expanding any section of the Scripture OS infrastructure.

#### **Section 1: Data Layer (The Physical Spine)**
* [ ] **PostgreSQL Extensions:** Ensure `ltree` and `pgcrypto` are active.
* [ ] **Migrations:** Create versioned SQL files for any schema changes instead of manual `init.sql` updates.
* [ ] **Specialized Indexing:**
    * [ ] GIST index for `path` (traversal).
    * [ ] B-Tree index for `absolute_index` (retrieval).
    * [ ] GIN index for `body_text` (search).
* [ ] **Constraint Enforcement:** Ensure `UNIQUE(edition_id, absolute_index)` to maintain structural integrity.

#### **Section 2: Service Layer (The Engines)**
* [ ] **Contract First:** Define the trait in `src/engines/mod.rs` before implementation.
* [ ] **Dependency Injection:** Use `Arc<dyn ScriptureRepository + Send + Sync>` for data access.
* [ ] **Micro-Documentation:**
    * [ ] Module-level `//!` with metaphor and "Why".
    * [ ] Function-level `///` with `Architectural Design Decision` and `AI Prompt Hint` headers.
* [ ] **Dual-Track Testing:**
    * [ ] `mod tests`: Integration with Postgres.
    * [ ] `mod mock_tests`: Isolated business logic with `MockRepository`.

#### **Section 3: Gateway Layer (The Interface)**
* [ ] **Type-Safe Routing:** Use Axum 0.7+ curly brace syntax (`{variable}`).
* [ ] **Payload Standardization:**
    * [ ] Wrap lists in `Pagination<T>`.
    * [ ] Group translations in `Comparison` objects.
* [ ] **Error Handling:** Map engine results to standardized JSON error objects with appropriate HTTP status codes.

#### **Section 4: Supporting Utilities**
* [ ] **Versification Mapper:** Implement logic for tradition-specific numbering overrides (e.g., Hebrew vs. English Psalms).
* [ ] **Ingestion Pipeline:** Build CLI tools to seed the database while maintaining the `ON CONFLICT` idempotency rule.