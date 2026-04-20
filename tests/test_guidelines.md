# Scripture OS: AI Testing Guidelines

When generating or modifying tests for Scripture OS (USDB V1), you MUST adhere to the following strict guidelines to prevent database collisions, race conditions, and flaky tests.

## 1. Use Deterministic, Hardcoded UUIDs
**NEVER use `Uuid::new_v4()` for test data insertion.**
Because tests run in parallel and database state persists between runs, random UUIDs will cause orphaned rows, duplicate keys, and foreign key violations.
* **Rule:** Always use `Uuid::parse_str("...")` with a recognizable, fixed pattern for the specific test module (e.g., `"11111111-1111-1111-1111-111111111111"`).

## 2. Isolate Test Data (Unique Slugs & Paths)
Because `cargo test` runs multi-threaded, two different test files inserting a work with the slug `'bible'` will cause a collision.
* **Rule:** Namespace your test data. If you are testing the Traversal engine, use slugs like `trav_test` and paths like `trav_test.book1`. If you are testing the Content engine, use `content_test`.

## 3. Ensure Idempotency (ON CONFLICT)
Tests must be able to run hundreds of times against the same local Docker database without failing.
* **Rule:** Every `INSERT` statement in a test MUST include `ON CONFLICT (id) DO NOTHING` or `ON CONFLICT (path) DO UPDATE SET...`.

## 4. Split Prepared Statements
By default, `sqlx::query()` uses prepared statements which do NOT allow multiple SQL commands separated by semicolons (`;`) in a single string.
* **Rule:** Break up multi-statement seeding scripts into individual `sqlx::query(...).execute(&pool).await.unwrap();` calls.

## 5. Use Shared Setup Utilities
Do not rewrite `setup_db()` or basic `seed_base_data()` logic in every test file.
* **Unit Tests (`src/`)**: Import from `crate::test_utils`.
* **Integration Tests (`tests/`)**: Import from `mod common;`.

## Example of a Perfect Test Setup:
```rust
#[tokio::test]
async fn test_example_feature() {
    let pool = crate::test_utils::setup_db().await;
    
    let work_id = Uuid::parse_str("99999999-9999-9999-9999-999999999999").unwrap();
    
    sqlx::query("INSERT INTO works (id, title, slug) VALUES ($1, 'Test', 'example_test') ON CONFLICT DO NOTHING")
        .bind(work_id)
        .execute(&pool)
        .await
        .unwrap();
        
    // ... run assertions ...
}