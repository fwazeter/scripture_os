# Scripture OS: Active Development Roadmap

This document tracks the immediate technical tasks required to fulfill the Scripture OS mission. Adhere to the `docs/guides/development_standards.md` for all implementations.

## 🟥 High Priority: Core Features

- [ ] **Implement Comparison Engine Logic**
    - [ ] Add `get_comparison(path: &str)` to the `ContentEngine` trait.
    - [ ] Implement logic to group `ScriptureContent` by `node_id` into the `Comparison` model.
    - [ ] Create Axum route: `GET /api/v1/compare/*path`.
- [ ] **Bootstrap Search Engine Infrastructure**
    - [ ] Define `SearchEngine` trait in `src/engines/mod.rs`.
    - [ ] Implement `PostgresRepository::search` using GIN indexes and Full-Text Search (FTS).
    - [ ] Implement `CoreSearchEngine` with support for `keyword_search` returning `Pagination<SearchMatch>`.

## 🟨 Medium Priority: Advanced Logic

- [ ] **Phase 3: Versification Mapper Utility**
    - [ ] Build standalone utility for tradition-based numbering overrides.
    - [ ] Integrate mapper into `ResolutionEngine` to align Tanakh and Bible Psalm numbering.
- [ ] **Regex Flexibility in Resolution Engine**
    - [ ] Update regex to support verse ranges (e.g., "3:16-17").
    - [ ] Update regex to support non-numeric indicators (e.g., "3:16a").

## 🟩 Low Priority: System Maturity

- [ ] **Traversal Pagination**
    - [ ] Update `get_hierarchy` to return `Pagination<HierarchyNode>` to prevent data flooding.
- [ ] **Domain-Specific Error Handling**
    - [ ] Replace generic `anyhow::Result` with a custom `ScriptureError` enum for better HTTP status codes.
- [ ] **Metadata Discovery Routes**
    - [ ] Implement endpoints to fetch detailed edition metadata (translators, language context).