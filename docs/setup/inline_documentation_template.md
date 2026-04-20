This template is designed to provide maximum context for both human developers and AI models. It prioritizes the **"Why"** (Architectural Intent) and the **"Gotchas"** (Quirks) over simple signatures, ensuring future AI prompts can maintain the system's integrity during refactors.

---

## Scripture OS Inline Documentation Template

```rust
    /// ## `{{function_name}}`
    /// **Parameters:** `{{param_name}}: {{type}}` ({{description}}).
    ///
    /// ### Architectural Design Decision: {{high_level_intent_title}}
    /// // Describe the high-level philosophy here. 
    /// // Explain what architectural problem this function solves (e.g., Stand-off Markup, Type Drift).
    /// 
    /// ### Design Decision: {{implementation_logic_title}}
    /// // Detail the specific logic used to achieve the intent.
    /// // 1. Itemize the steps.
    /// // 2. Explain why this specific algorithm/index was chosen.
    ///
    /// ### {{Technical_Context_Type}}: {{specific_detail_title}}
    /// // (e.g., SQL Quirk, Rust Lifecycle, Performance Constraint).
    /// // Explain specific non-obvious details like LTREE casting, CTE strategies, 
    /// // or pointer management.
    ///
    /// **AI Prompt Hint:** {{instruction_for_future_ai}}
    /// // Explicitly tell future AI models how to modify this code or 
    /// // what edge cases to look for (e.g., "If adding feature X, update query Y").
    async fn {{function_name}}(&self, {{params}}) -> Result<{{ReturnType}}> {
        // ... implementation ...
    }
```

---

### Example Applied to `get_hierarchy`:

To see how this works in practice, here is a breakdown of another core function using the template:

```rust
    /// ## `get_hierarchy`
    /// **Parameters:** `parent_path: &str` (The canonical LTREE path of the parent node).
    ///
    /// ### Architectural Design Decision: Depth-Limited Exploration
    /// Prevents "Data Flooding" by ensuring that a request for a high-level node (e.g., a "Work") 
    /// doesn't accidentally return every verse within it.
    /// 
    /// ### Design Decision: Structural Integrity
    /// Uses the `nlevel()` PostgreSQL function to calculate direct lineage.
    /// 1. Identifies the depth of the `parent_path`.
    /// 2. Filters for nodes exactly one level deeper (`nlevel + 1`).
    ///
    /// ### SQL Quirk: Double Casting for LTREE
    /// Because `sqlx` does not have a native `ltree` type, we must cast the input string 
    /// to `text` first, and then to `ltree` (`$1::text::ltree`) to ensure the GIST 
    /// index is utilized correctly by the query optimizer.
    ///
    /// **AI Prompt Hint:** If implementing "Deep Retrieval" or "Recursive Menus," 
    /// remove the `nlevel` constraint while keeping the `<@` (descendant) operator.
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>> {
        // ... implementation ...
    }
```