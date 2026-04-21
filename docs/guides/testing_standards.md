# Scripture OS: Testing Standards

Adherence to these standards ensures systemic integrity and high-quality AI-assisted development.

## 1. Use Deterministic, Hardcoded UUIDs
**NEVER use `Uuid::new_v4()` for test data insertion.** Random UUIDs cause collisions in parallel test environments.
* **Rule:** Use `Uuid::parse_str("...")` with fixed patterns for specific modules.

## 2. Isolate Test Data
* **Rule:** Namespace test data with unique slugs (e.g., `search_test_work`) to prevent collisions during multi-threaded `cargo test` runs.

## 3. Ensure Idempotency (ON CONFLICT)
* **Rule:** Every `INSERT` must include `ON CONFLICT (id) DO NOTHING` or similar clauses to ensure tests can run repeatedly against a persistent database.

## 4. Split Prepared Statements
* **Rule:** Break up multi-statement seeding scripts into individual `sqlx::query` calls, as prepared statements do not allow semicolon-separated commands.

## 5. Use Shared Setup Utilities
* **Rule:** Reuse `crate::test_utils::setup_db()` and avoid rewriting boilerplate setup logic in individual files.

## 6. Resilient Text Assertions (New)
Database-generated content (like search snippets or stemmed text) is often non-deterministic in its exact formatting or casing.
* **Rule:** Avoid strict string equality (`assert_eq!`) for search snippets.
* **Rule:** Use partial matches (`assert!(snippet.contains("<b>"))`) to verify that functionality (like HTML highlighting) is active without failing on minor formatting quirks.