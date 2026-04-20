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
