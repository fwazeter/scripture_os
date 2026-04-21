# Scripture OS: Active Development Roadmap

This document tracks the immediate technical tasks required to fulfill the Scripture OS mission. Adhere to the `docs/guides/development_standards.md` for all implementations.

## 🟥 High Priority: Core Features

- [X] **Implement Comparison Engine Logic**
    - [X] Add `get_comparison(path: &str)` to the `ContentEngine` trait.
    - [X] Implement logic to group `ScriptureContent` by `node_id` into the `Comparison` model.
    - [x] Create Axum route: `GET /api/v1/compare/*path`.
    - [x] Dual-Track Testing Compliance.
      - [x] Added `mod mock_tests` for the Content Engine to verify grouping logic in isolation.
      - [x]Updated `mod tests` with integration tests for `get_comparison`.
- [X] **Bootstrap Search Engine Infrastructure**
    - [x] Define `SearchEngine` trait in `src/engines/mod.rs`.
    - [x] Implement `PostgresRepository::search` using GIN indexes and Full-Text Search (FTS).
    - [x] Implement `CoreSearchEngine` with support for `keyword_search` returning `Pagination<SearchMatch>`.
    - [x] ADd global integration tests in `tests/search_test.rs`.

## 🟨 Medium Priority: Advanced Logic

- [ ] **Phase 3: Versification Mapper Utility**
    - [ ] Build standalone utility for tradition-based numbering overrides.
    - [ ] Integrate mapper into `ResolutionEngine` to align Tanakh and Bible Psalm numbering.
- [ ] **Regex Flexibility in Resolution Engine**
    - [ ] Update regex to support verse ranges (e.g., "17:2-3").
    - [ ] Update regex to support non-numeric indicators (e.g., "17:3a").

## 🟩 Low Priority: System Maturity

- [ ] **Traversal Pagination**
    - [ ] Update `get_hierarchy` to return `Pagination<HierarchyNode>` to prevent data flooding.
- [ ] **Domain-Specific Error Handling**
    - [ ] Replace generic `anyhow::Result` with a custom `ScriptureError` enum for better HTTP status codes.
- [ ] **Metadata Discovery Routes**
    - [ ] Implement endpoints to fetch detailed edition metadata (translators, language context).