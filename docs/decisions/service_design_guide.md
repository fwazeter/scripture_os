# Scripture OS: Service Layer Design & Documentation Guide

This guide establishes the architectural and documentation standards for developing "Engines" (Service Layer modules) within Scripture OS. Both human developers and AI coding assistants must strictly adhere to these patterns to ensure modularity, testability, and clear communication of intent.

## 1. Core Architectural Philosophy

Scripture OS uses a **Trait-Based, Dependency-Injected Architecture**.
We separate the "Contract" (what a service does) from the "Implementation" (how it does it) and the "Dependencies" (what it relies on).

### Key Rules for AI & Human Developers:
1. **Contract-First:** Never write a standalone public function for business logic. Define a trait first (e.g., `TraversalEngine`).
2. **Dependency Injection (DI):** Engines must not instantiate their own database connections. Dependencies must be injected via `Arc<dyn Trait + Send + Sync>`.
3. **Document the "Why":** Standard Rust documentation tells you *what* the code does. Scripture OS documentation must explicitly state *why* it does it using `Architectural Design Decision` headers.

---

## 2. Module & Struct Layout Pattern

Every engine must be implemented as a stateful struct that implements an `#[async_trait]`.

### The Struct Definition
Use the `Core[Name]Engine` naming convention. The struct must encapsulate its dependencies in an `Arc` to allow safe sharing across concurrent Axum web requests.

```rust
use std::sync::Arc;
use async_trait::async_trait;
use crate::repository::ScriptureRepository;

/// # Core [Name] Engine
///
/// This is the primary implementation of the `[Name]Engine` trait.
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// This struct holds a thread-safe, reference-counted pointer (`Arc`) to the underlying
/// data repository. This allows the engine to manage its own state and be easily 
/// shared across concurrent asynchronous tasks in the Axum web framework.
pub struct CoreMyServiceEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreMyServiceEngine {
    /// Bootstraps the engine by injecting the required data layer repository.
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}
```

---

## 3. The Documentation Standard

This is the most critical section for AI replication. Every module and function must follow this exact markdown structure inside the Rust doc comments (`///` or `//!`).

### Module-Level Documentation (`//!`)
Every engine file must begin with a high-level explanation of its role, metaphor, and primary design decisions.

**Template:**
```rust
//! # [Engine Name] Engine (The "[Metaphor]")
//!
//! [1-2 sentences explaining the high level responsibility of the engine].
//!
//! ### Architectural Design Decision: [Name of Concept]
//! [Explain the architectural boundary. E.g., "Scripture OS separates Addressing from Content. 
//! This engine is concerned exclusively with Addressing..."]
```

### Function-Level Documentation (`///`)
Functions must document their parameters, their design justification, and any relevant technical context.

**Template:**
```rust
    /// ## `[method_name]`
    /// **Parameters:** /// * `[param_name]: [type]` ([Brief explanation of the parameter]).
    ///
    /// ### Architectural Design Decision: [Name of Decision]
    /// [Explain WHY this function exists and WHY it behaves the way it does. 
    /// E.g., "This function enables the UI to load scripture in chunks rather than 
    /// downloading the entire hierarchy at once..."]
    ///
    /// ### Design Decision: Engine-to-Repo Delegation (If applicable)
    /// [Explain how business logic is separated from data fetching].
```

---

## 4. Implementation Rules: Delegation over Duplication

Engines are orchestrators. They apply business logic but should not write raw SQL or handle complex data aggregation if the database can do it faster.

* **Validate then Delegate:** The engine should validate inputs, enforce formatting, and apply business rules. It should then delegate the specific filtering/querying logic to the injected repository.
* **Example:** In the Traversal Engine, the engine decides *what* constitutes a hierarchy request, but calls `self.repo.get_hierarchy(parent_path)` to actually fetch the LTREE nodes.

---

## 5. Testing Paradigm: Dual-Track Verification

Because we use Dependency Injection, every engine must include two testing modules in the same file.

### Track A: Concrete Integration Tests (`mod tests`)
Tests how the engine interacts with the real database implementation.
* Must use `crate::test_utils::setup_db()` and seed data.
* Must inject `PostgresRepository`.

### Track B: Isolated Mock Tests (`mod mock_tests`)
Tests the engine's internal business logic in isolation without touching a database.
* Must define a `struct MockRepository;` that implements the target trait.
* Must return hardcoded `Ok()` responses to verify that the engine correctly routes data.

**Mock Implementation Pattern:**
```rust
#[cfg(test)]
mod mock_tests {
    use super::*;
    // ... imports ...

    struct MockRepository;

    #[async_trait]
    impl ScriptureRepository for MockRepository {
        async fn some_method(&self) -> Result<SomeData> {
            Ok(SomeData { /* fake data */ })
        }
        // stub other methods...
    }

    #[tokio::test]
    async fn test_engine_logic_with_mock() {
        let repo = Arc::new(MockRepository);
        let engine = CoreMyServiceEngine::new(repo);
        
        let result = engine.some_method().await.unwrap();
        assert!(/* verify engine logic */);
    }
}
```

---

### **System Prompt Addendum for AI:**
*When prompted to generate a new service, engine, or module for Scripture OS, you MUST read this document and implement the `Core[Name]Engine` struct, exact DI injection patterns, exact `Architectural Design Decision` doc-comment headers, and both `mod tests` and `mod mock_tests` as defined above.*