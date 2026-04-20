# Scripture OS: Documentation Guide

Because Scripture OS deals with highly abstract concepts (Stand-off Markup, LTREE hierarchies, and parallel sequencing), rigorous documentation is critical. This project utilizes a **Dual-Layer Documentation Strategy**:
1. **Macro Documentation:** Markdown files detailing systemic architecture.
2. **Micro Documentation:** In-line Rustdoc detailing programmatic implementation.

---

## 1. Project Folder Structure

To keep the repository clean, all architectural and setup documentation must be placed within the `docs/` folder. The root directory should only contain the `README.md` and build files.

```text
scripture_os/
├── docs/                       
│   ├── architecture/
│   │   ├── database_schema.md  <-- Core ER diagrams and table definitions
│   │   └── stand_off_markup.md <-- Explanations of index mapping concepts
│   ├── api/
│   │   └── endpoints.md        <-- REST/GraphQL route specifications
│   └── setup/
│       ├── docker_guide.md     <-- Database initialization instructions
│       └── documentation_guide.md <-- THIS FILE
├── src/                        <-- Rust Source Code (Micro Documentation)
├── tests/                      <-- Integration Tests
├── Cargo.toml                  
├── docker-compose.yml          
└── README.md                   <-- Primary entry point linking to docs/