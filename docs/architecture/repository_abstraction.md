# The Data Access Layer (DAL)
The Repository Pattern serves as the "Interface" between the logical requirements of the Scripture OS engines and the physical storage in PostgreSQL.

##  Repository Pattern & Data Abstraction

### Goals of this Abstraction
1.  **Strict Contract Enforcement**: The `ScriptureRepository` trait defines exactly what data the system needs, regardless of where it comes from.
2.  **Implementation Hiding**: The Engines (Traversal, Content, Resolution) no longer know about `ltree` operators, `BETWEEN` joins, or `ILIKE` matches. They simply request domain models.
3.  **Mockability**: By using a trait, we can perform unit tests on engine logic (like regex parsing or path validation) using an in-memory mock repository without requiring a live database connection.

### Implementation Details
* **Postgres Implementation**: Uses `sqlx` and takes advantage of PostgreSQL's specialized `ltree` type for hierarchy.
* **Trait Composition**: Uses the `async_trait` crate to handle asynchronous methods within the trait definition.
* **Dependency Injection**: The repository is wrapped in an `Arc` and injected into the Axum web state, allowing thread-safe access across all API routes.

## The Core Abstraction: `ScriptureRepository`
The system no longer passes `sqlx::PgPool` directly into core functions. Instead, it passes a reference to `dyn ScriptureRepository`.

### Why this change was made:
1.  **Testability:** We can now test engine logic (like complex path parsing or traversal sorting) using a `MockRepository` without needing a running database.
2.  **Database Agnosticism:** The core engines are now "blind" to the storage engine. We can swap PostgreSQL for SQLite (for offline/mobile use) or a Graph Database without changing a single line of code in the engines.
3.  **Separation of Concerns:** `traversal.rs` and `content.rs` now focus exclusively on *what* to do with data, while `postgres.rs` focuses on *how* to fetch it using SQL and `ltree` operators.

## System Flow
1.  **Request:** A user hits `/v1/read/bible?q=John 3:16`.
2.  **Handler:** The Axum handler extracts the `Arc<dyn ScriptureRepository>` from the state.
3.  **Resolution Engine:** Converts "John 3:16" to `bible.nt.john.3.16` by calling `repo.resolve_address()`.
4.  **Content Engine:** Fetches text segments by calling `repo.fetch_text()`.
5.  **Response:** The handler packages the results into JSON.
    Following the **Dual-Layer Documentation Strategy**, here is the comprehensive documentation for the Scripture OS Repository Refactor. This moves low-level SQL implementation details into the Repository layer and maintains high-level business logic in the Engines.

This documentation outlines the architectural shift from direct database calls to a **Repository Abstraction Layer** within Scripture OS. This refactor decouples the business logic (Engines) from the persistence layer (Postgres), enabling better testability and modularity as defined in the **Development Plan**.

---

# Scripture OS: Repository Abstraction Documentation

## 1. `src/repository/mod.rs`
**Role:** The "Contract" Layer.
This file defines the traits that specify how the system interacts with data, regardless of the underlying storage engine. It serves as the interface for the Resolution, Content, and Traversal engines.

### Defined Traits
| Trait | Purpose | Key Methods |
| :--- | :--- | :--- |
| **`ScriptureRepo`** | Master trait for data access. | `resolve_path()`, `fetch_text()`, `get_adjacents()`. |
| **`ContentProvider`** | Interface for text retrieval. | `get_verses_by_path()`, `get_translations()`. |
| **`HierarchyProvider`** | Interface for structural lookups. | `get_children()`, `find_node_by_alias()`. |

**Implementation Strategy:**
- Utilizes `async_trait` to handle asynchronous database interactions.
- Returns `Result<T, RepoError>` to abstract database-specific errors (like `sqlx::Error`) into domain-specific errors.

---

## 2. `src/repository/postgres.rs`
**Role:** The "Implementation" Layer.
This file provides the concrete implementation of the repository traits using **SQLx** and **PostgreSQL**. It encapsulates the specialized `ltree` logic and complex joins defined in the **Database Schema**.

### Key Implementations
* **Path Resolution:** Implements logic to query the `nodes` table using `ltree` operators (e.g., `~` for matches).
* **Content Retrieval:** Houses the refactored `get_verses_by_path` logic originally found in `lib.rs`, utilizing the `JOIN` between `texts`, `nodes`, and `editions`.
* **Performance:** Implements the indexing strategy (GIST for `ltree`, B-Tree for `sort_order`) within the query logic to ensure rapid sequential retrieval.

---

## 3. `src/resolution.rs`
**Role:** The "Router" (Module 1).
This file implements the **Resolution Engine**. It is responsible for transforming human-readable input into validated system addresses (`ltree` paths).

### Responsibilities
* **`parse_address(input)`**: Uses regex and the `node_aliases` table to convert strings like "Jn 3:16" into the canonical `bible.nt.john.3.16` path.
* **`resolve_path(path)`**: Validates the existence of a path against the `nodes` table via the `HierarchyProvider` trait.
* **Dependency Injection**: This module accepts any object implementing `HierarchyProvider`, making it independent of Postgres for unit testing.

---

## 4. `src/content.rs`
**Role:** The "Assembler" (Module 2).
This file implements the **Content Engine**, focusing on retrieving, aggregating, and formatting scriptural text for the API.

### Responsibilities
* **`fetch_content(path, edition_ids)`**: Orchestrates the retrieval of text. If no `edition_ids` are provided, it defaults to the `is_primary` edition for the associated work.
* **`compare_translations(node_id, editions)`**: Logic for side-by-side comparison of multiple translations (e.g., comparing KJV and SBLGNT).
* **Data Transformation**: Maps the raw `ScriptureContent` database rows into JSON structures ready for the Axum handlers.

---

## 5. `src/traversal.rs`
**Role:** The "Guide" (Module 4).
This file implements the **Navigation Engine**, enabling users to move through the scripture sequentially or hierarchically.

### Responsibilities
* **`get_adjacent_nodes(current_path)`**: Queries the repository for the "Previous" and "Next" nodes based on the `sort_order` and `node_type` (e.g., moving from Chapter 2 to Chapter 3).
* **`get_hierarchy(parent_path)`**: Fetches all child nodes of a given path (e.g., listing all books in the "New Testament" tradition).


---

## Summary of Data Flow
1.  **Request**: User hits `GET /v1/read/john.3.16`.
2.  **Resolution**: `resolution.rs` validates the path against the repository.
3.  **Content**: `content.rs` calls the repository to fetch the text and edition metadata.
4.  **Response**: The Engine formats the result into the `ScriptureContent` model and returns it via the API.