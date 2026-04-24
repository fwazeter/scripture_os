//! # Scripture OS Core
//!
//! ### Architectural Design Decision: Domain-Driven Layering
//! This project is organized into vertical layers:
//! 1. **FSI**: The immutable "DNA" and coordinate system.
//! 2. **Repository**: The abstract data access layer (DAL).
//! 3. **Engines**: The service layer where business logic resides.
//! 4. **Utils**: Shared primitives like error handling.

pub mod api;
pub mod engines;
pub mod fsi;
pub mod lexicon;
pub mod parsers;
pub mod repository;
pub mod utils;
