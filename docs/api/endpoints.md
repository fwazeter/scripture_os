# API Endpoints & Payload Specifications

## 1. Overview
The Gateway Layer is powered by **Axum** and acts as a type-safe interface for the underlying Service Engines. Every endpoint follows a standard response pattern, ensuring consistency for automated consumption.

### **Architectural Design Decision: Gateway Abstraction**
The API does not talk to the database. It interprets HTTP requests, invokes the relevant `Engine` trait, and serializes the resulting `models` into JSON. This allows us to change the database or engine implementation without breaking the external contract.

---

## 2. Core Endpoints

### **A. Resolve Shorthand Address**
Translates human shorthand (e.g., "Jn 17:3") into a canonical LTREE path.

* **URL:** `GET /api/v1/resolve/{work_slug}/{address}`
* **Parameters:**
    * `work_slug`: The corpus identifier (e.g., `bible`).
    * `address`: The raw input string (e.g., `Jn 17:3`).
* **Success Response (JSON):**
    ```json
    { "data": {
      "start_path": "bible.nt.john.17.3",
      "end_path": "bible.nt.john.17.5"
    } 
  }
    ```

### **B. Fetch Content**
Retrieves aligned text segments for a specific coordinate.

* **URL:** `GET /api/v1/content/{*path}`
* **Parameters:**
    * `path`: The canonical LTREE address (e.g., `bible.nt.john.17.3`).
    * `end_path` *(Optional Query Param)*: The canonical LTREE address to end retrieval (e.g. `?end_path=bible.nt.john.17.5`).
* **Success Response (JSON):**
    ```json
    {
      "data": [
        {
          "node_id": "uuid-v4-string",
          "path": "bible.nt.john.17.3",
          "body_text": "For God so loved the world...",
          "edition_name": "KJV",
          "language_code": "en",
          "absolute_index": 4000,
          "translation_metadata": null
        }
      ]
    }
    ```

### **C. Compare Content**
Groups multiple translations under their shared structural coordinate for side-by-side parallel reading.

* **URL:** `GET /api/v1/compare/{*path}?end_path=...`
* **Parameters:**
  * `path`: The canonical LTREE address to begin comparison.
  * `end_path` *(Optional Query Param)*: The canonical LTREE address to end comparison.
* **Success Response (JSON):**
    ```json
    {
      "data": [
        {
          "node_id": "uuid",
          "path": "canonical.path",
          "contents": [
            { "edition_name": "Translation_A", "body_text": "..." },
            { "edition_name": "Translation_B", "body_text": "..." }
          ]
        }
      ]
    }
    ```

### **D. Explore Hierarchy**
Provides structural discovery for building menus or navigation trees.

* **URL:** `GET /api/v1/hierarchy/{*path}`
* **Parameters:**
  * `path`: The parent node path (e.g., `bible.nt.john`).
* **Success Response (JSON):**
    ```json
    {
      "data": [
        { "id": "uuid-v4-string", "path": "bible.nt.john.1" },
        { "id": "uuid-v4-string", "path": "bible.nt.john.2" }
      ]
    }
    ```

---

## 3. Advanced Payload Models

### **The `Comparison` Object**
Used for parallel reading views where multiple translations are aligned under a single structural coordinate.

### **Architectural Design Decision: Node-Centric Comparison**
To facilitate side-by-side analysis, the API groups `ScriptureContent` units by their shared `node_id`. This structure is optimized for AI digestion, as it explicitly links disparate linguistic strings to the same semantic anchor point.

* **Payload Structure:**
    ```json
    {
      "node_id": "uuid",
      "path": "canonical.path",
      "contents": [
        { "edition_name": "Translation_A", "body_text": "..." },
        { "edition_name": "Translation_B", "body_text": "..." }
      ]
    }
    ```

### **The `Pagination<T>` Wrapper**
Standardizes all list-based responses to ensure predictable consumption by frontend state managers and AI iterators.

* **Payload Structure:**
    ```json
    {
      "data": [...],
      "total_records": 100,
      "current_page": 1,
      "total_pages": 5,
      "has_next": true
    }
    ```
---

## 4. Error Handling
Scripture OS uses standard HTTP status codes and a standardized error object:
```json
{ "error": "Book alias 'NonExistent' not found" }
```

**AI Prompt Hint:** If you are implementing a new endpoint for "Search," ensure it returns the `Pagination<SearchMatch>` model to maintain consistency with existing discovery patterns.

This Stage 5 document, `docs/api/endpoints.md`, defines the formal specification for the **Gateway Layer**. It ensures that external clients—ranging from mobile frontends to AI agents—interact with the Scripture OS engines using predictable, context-rich data structures.
