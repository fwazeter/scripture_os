# Architecture: Stand-off Markup & Parallel Sequencing

## 1. The Core Philosophy
Scripture OS utilizes a **Stand-off Markup** architecture. Unlike traditional formats (like XML or JSON-nested scripture) where text is wrapped in structural tags, Scripture OS completely decouples the **Hierarchy (The Spine)** from the **Linguistic Content (The Muscle)**.

### **Architectural Design Decision: Structural Decoupling**
By separating "Address" from "Text," we solve the problem of "Overlapping Hierarchies." This allows a single verse to be part of multiple organizational systems (e.g., a theological system like *Mandala* and a memorization system like *Ashtaka*) without duplicating the actual text in the database.

---

## 2. The Two-Pillar Model

The system relies on two primary tables in PostgreSQL to maintain this separation:

### **Pillar A: The Structural Spine (`nodes` table)**
The `nodes` table defines the canonical address space using the `ltree` extension.
* **Path:** A hierarchical address (e.g., `bible.nt.john.17.3`).
* **Sequence Pointers:** Every node contains a `start_index` and an `end_index`. These are not text; they are integers that define a "window" into the universal sequence.

### **Pillar B: The Content Layer (`texts` table)**
The `texts` table stores the actual strings of scripture.
* **Absolute Index:** A unique integer representing a specific "slot" in a linear sequence.
* **Edition Link:** Every text row is linked to an `edition_id` (e.g., KJV, NIV, SBLGNT).
* **Body Text:** The raw linguistic content.

---

## 3. The Bridge: Universal Indexing

The "Bridge" between these two pillars is the `absolute_index`.



### **How Retrieval Works (The "Window" Strategy)**
When a user requests a node (e.g., "John 17:3"), the **Content Engine** performs a two-step resolution:
1. **Find the Window:** It asks the `nodes` table for the `start_index` and `end_index` of the path `bible.nt.john.17.3`.
2. **Collect the Content:** It queries the `texts` table for all rows where the `absolute_index` falls **BETWEEN** those two numbers.

### **Design Decision: Parallel Alignment**
Because different editions (translations) use the same `absolute_index` for the same semantic unit, a single query automatically returns every available translation for that node, perfectly aligned for comparison.

---

## 4. Technical Benefits for AI & Developers

### **Architectural Design Decision: Context Injection**
Because the text is "ignorant" of its path, the system injects the `node_id` and `path` metadata at the moment of retrieval. This ensures that when data is fed into an AI model, the model receives the text *and* its exact coordinates, eliminating hallucinations regarding the text's location.

### **Performance: Index Optimization**
* **Hierarchical Queries:** Powered by GIST indexes on the `ltree` path.
* **Range Queries:** Powered by standard B-tree indexes on `absolute_index`, allowing for lightning-fast retrieval of large sections (entire books or testaments).

---

**AI Prompt Hint:** If you are asked to implement a new "interlinear" view, you do not need to change the schema. Simply fetch all texts for a range of `absolute_index` values and group them by that index in your application logic.
