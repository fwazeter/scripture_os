# Architecture: The Data Access Layer (DAL)

The Repository Pattern serves as the "Physical Interface" between the high-level business logic of the Scripture OS engines and the underlying storage in PostgreSQL.

### **Architectural Design Decision: The Repository Abstraction**
Scripture OS decouples the **Service Layer** (Engines) from the **Data Layer** (Postgres) using the `ScriptureRepository` trait.
* **Implementation Hiding**: Engines no longer know about SQL syntax, `ltree` operators, or database-specific connection pools.
* **Strict Contract Enforcement**: The trait defines exactly what data the system needs, ensuring any future data source (e.g., SQLite or a remote API) can be swapped in without breaking business logic.
* **Testability**: By using a trait, we can perform unit tests on engine logic using a `MockRepository` that returns hardcoded data, eliminating the need for a live database during logic verification.

---

## 1. The Core Abstraction: `ScriptureRepository`

The system utilizes the `async_trait` crate to define a thread-safe, asynchronous contract for data retrieval.

### **Key Responsibilities**
* **Address Resolution**: Mapping human shorthands to canonical paths via alias lookups.
* **Hierarchical Discovery**: Querying the tree for parents, children, and siblings.
* **Content Retrieval**: Fetching sequential text segments based on sequence boundaries defined in the spine.

### **Technical Context: Dependency Injection (DI)**
Engines do not instantiate the repository. Instead, an `Arc<dyn ScriptureRepository + Send + Sync>` is injected into the engine during bootstrap. This allows a single `PostgresRepository` instance to be shared across all engines in the Axum `AppState`.

---

## 2. Physical Implementation: `PostgresRepository`

This is the concrete implementation of the repository trait optimized for PostgreSQL.

### **Architectural Design Decision: The Spine and Muscle Model**
The repository manages the relationship between two distinct tables:
1.  **`nodes` (The Spine)**: Stores the hierarchical path (`ltree`) and the sequential bounds (`start_index`, `end_index`).
2.  **`texts` (The Muscle)**: Stores the linguistic content indexed by `absolute_index`.

### **Technical Context: SQL Efficiency**
* **Single-Trip Queries**: Complex operations like adjacency (Next/Previous) are handled using Common Table Expressions (CTEs) to perform anchor lookups and neighbor scans in one database round-trip.
* **LTREE Optimization**: The repository handles the double-casting (`$1::text::ltree`) required to ensure PostgreSQL correctly utilizes GIST indexes for path queries.

---

## 3. Interaction with Engines

The Engines act as orchestrators that consume the repository's output.

| Engine | Role | Primary Repository Interaction |
| :--- | :--- | :--- |
| **Resolution** | The "Router" | Calls `resolve_address` to convert shorthand into LTREE paths. |
| **Content** | The "Assembler" | Calls `fetch_text` to bridge paths back to sequential text segments. |
| **Traversal** | The "Guide" | Calls `get_hierarchy` and `get_adjacent_nodes` for discovery. |

---

## 4. Testing with the Repository

### **Track A: Integration Testing**
Integration tests utilize `test_utils::setup_db()` to create a real Postgres instance, which is then wrapped in an `Arc` and injected into the engine.

### **Track B: Mock Testing**
Mock tests define a `MockRepository` struct within the engine's test module. This allows for testing edge cases—like malformed regex in the Resolution Engine—without the overhead of database setup.

**AI Prompt Hint:** If you are asked to implement a new data provider (e.g., a "MemoryRepository"), you must implement the `ScriptureRepository` trait and use `parking_lot::RwLock` for thread-safe internal state management to ensure compatibility with the existing engine DI pattern.
