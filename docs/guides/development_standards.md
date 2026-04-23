# Scripture OS: Development & Documentation Standards

This document establishes the universal standards for code structure, architectural philosophy, and documentation within the Scripture OS project. Adherence to these standards ensures systemic integrity, modularity, and high-quality AI-assisted development.

> **The Design Mandate:** Every module, struct, and function must prioritize the **"Why"** (Architectural Intent) over the **"What"** (Implementation).

---

## 1. Core Architectural Philosophy

Scripture OS utilizes a **Trait-Based, Dependency-Injected (DI) Architecture**. This separates the "Contract" from the "Implementation" and its external "Dependencies".

### **The Three Golden Rules**
1.  **Contract-First:** Define a trait (e.g., `TraversalEngine`) in `src/engines/mod.rs` before writing any business logic.
2.  **Dependency Injection:** Engines must not instantiate their own database pools or external services. Inject dependencies via `Arc<dyn Trait + Send + Sync>`.
3.  **Document the Intent:** Every non-trivial block of code must use `Architectural Design Decision` headers to explain the underlying philosophy.

---

## 2. Project Folder Structure

To maintain a clean repository, all macro-level documentation and setup guides are stored in the `docs/` folder. The root directory is reserved for build files and the primary `README.md`.

```text
scripture_os/
├── docs/                       
│   ├── architecture/           <-- Systemic design (LTREE, Stand-off Markup)
│   ├── api/                    <-- Route specifications and payloads
│   ├── guides/                 <-- Procedural standards (THIS FILE)
│   ├── templates/              <-- Blueprints for AI code generation
│   └── decisions/              <-- Historical Design Decision Logs (ADR)
├── src/                        <-- Rust Source Code (Micro Documentation)
│   ├── engines/                <-- Service Layer Logic
│   ├── repository/             <-- Physical Data Layer (Postgres)
│   └── models.rs               <-- Central Data Contracts
├── tests/                      <-- Cross-module Integration Tests
└── README.md                   <-- Primary entry point

src/
├── main.rs                 <-- Application entry point
├── lib.rs                  <-- Module declarations (pub mod fsi; pub mod repository; etc.)
│
├── fsi/                    <-- DOMAIN: The "Stable Bottom" (FSI v4.0)
│   ├── mod.rs              <-- Exports FSI components
│   ├── models.rs           <-- The 5-part Coordinate, WorkID, SubMask, etc.
│   └── lex_key.rs          <-- Base-62 string generation & comparison utility
│
├── repository/             <-- DOMAIN: The Data Access Layer (DAL)
│   ├── mod.rs              <-- Defines the `ScriptureRepository` trait contract
│   ├── postgres.rs         <-- Concrete FSI implementation using sqlx
│   └── mock.rs             <-- In-memory mock for Track B testing
│
├── engines/                <-- DOMAIN: The "Muscle" (Service Layer)
│   ├── mod.rs              
│   ├── content.rs          <-- Assembles FSI fragments into readable text
│   ├── resolution.rs       <-- Translates human shorthands to FSI coordinates
│   └── traversal.rs        <-- Navigates the FSI trees (next/prev/children)
│
├── lenses/                 <-- DOMAIN: The "Liquid Top" (WASM Plugins)
│   ├── mod.rs              <-- Defines the `Lens` trait contract
│   └── wasm_host.rs        <-- Wasmtime integration for external logic
│
└── utils/                  <-- DOMAIN: Shared/Generic helpers
    ├── mod.rs
    └── errors.rs           <-- Domain-specific `ScriptureError` enum
```


---

## 3. Module & Struct Layout Pattern

Every service layer engine must be implemented as a stateful struct named with the `Core[Name]Engine` convention.

### **The Engine Blueprint**
```rust
use std::sync::Arc;
use async_trait::async_trait;
use crate::repository::ScriptureRepository;

/// # Core [Name] Engine
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// This struct encapsulates a thread-safe `Arc` to the repository, allowing 
/// for modular state management and cross-thread safety in Axum.
pub struct CoreMyServiceEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreMyServiceEngine {
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}
```


---

## 4. Documentation Standards (Micro-Docs)

Documentation is critical for future refactors and AI context. Every module and function must follow this structure within Rustdoc comments.

### **Module-Level (`//!`)**
Explain the high-level responsibility and the primary metaphor (e.g., "The Guide", "The Assembler").
```rust
//! # [Engine Name] Engine (The "[Metaphor]")
//!
//! ### Architectural Design Decision: [Name of Concept]
//! [Explain the architectural boundary, e.g., "Decoupling addressing from content."]
```

### **Function-Level (`///`)**
Prioritize the philosophy and technical quirks over the signature.
```rust
/// ## `[method_name]`
/// **Parameters:** `[param]: [type]` ([Description]).
///
/// ### Architectural Design Decision: [Title]
/// [Explain WHY this function exists and the problem it solves].
///
/// ### Design Decision: [Logic_Title] (Optional)
/// [Explain specific algorithms or why a certain index was chosen].
///
/// ### Technical Context: [Detail_Title] (e.g., SQL Quirk)
/// [Explain non-obvious details like LTREE casting or CTE strategies].
///
/// **AI Prompt Hint:** [Explicit instruction for future modifications].
```

---

## 5. Testing Paradigm: Dual-Track Verification

All engines must be verified using two distinct tracks within the same file.

1.  **Track A: Concrete Integration Tests (`mod tests`)**: Tests interaction with the real PostgreSQL implementation via `test_utils::setup_db()`.
2.  **Track B: Isolated Mock Tests (`mod mock_tests`)**: Tests business logic in total isolation by injecting a `struct MockRepository`.

---

## 6. System Prompt Addendum for AI Agents

**Attention AI Agent:** When prompted to generate a new service, engine, or module for Scripture OS, you **MUST** follow these steps:
1.  Read this `development_standards.md` file in full.
2.  Implement the `Core[Name]Engine` struct with the exact DI patterns specified.
3.  Use the `Architectural Design Decision` and `AI Prompt Hint` headers for all public functions.
4.  Generate both `mod tests` and `mod mock_tests` in the resulting implementation.
