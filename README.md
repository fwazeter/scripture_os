# Scripture OS

**Scripture OS** is a high-performance, future-proof architectural framework built in Rust for managing, comparing, and discovering the world's scriptural wisdom. It is designed with a "contract-first" approach, utilizing a trait-based Dependency Injection (DI) architecture to facilitate AI digestion and advanced comparison logic.

## 📖 Mission and Purpose
The mission of Scripture OS is to provide a standardized, scalable infrastructure for the world's scriptures.
* **Universal Access:** To maintain a modular library of global scripture in a way that respects original traditions while enabling cross-tradition comparison.
* **AI-First Design:** To provide contextually enriched data (paths, metadata, and hierarchy) that allows AI models to analyze scripture without hallucinations.
* **Structural Integrity:** Using a "Stand-off Markup" model, the system decouples linguistic content from hierarchical structure, allowing for multiple translations and overlapping traditions to coexist on a single canonical spine.

## 🏗️ Integrated Architecture
The system is divided into three distinct layers:
1. **Data Layer:** PostgreSQL utilizing the `ltree` extension for hierarchical path management and `absolute_index` for linear sequence.
2. **Service Layer (Engines):** Trait-based engines (Content, Resolution, Traversal) that handle business logic through Dependency Injection.
3. **Gateway Layer:** An Axum-powered REST API that exposes the engines to users and AI programs.

## 🚀 Installation

### Prerequisites
* **Rust:** Latest stable version (Edition 2024 recommended).
* **Docker:** For running the PostgreSQL instance.

### Setup Steps
1. **Clone the Repository:**
   ```bash
   git clone https://github.com/fwazeter/scripture_os.git
   cd scripture_os
   ```
2. **Environment Configuration:**
   Copy the `.env-sample` to `.env` and configure your database URL.
   ```bash
   cp .env-sample .env
   ```
3. **Start the Database:**
   ```bash
   docker-compose up -d
   ```
4. **Run the API:**
   ```bash
   cargo run
   ```

## 🛠️ Usage
Scripture OS provides specialized endpoints for navigating and retrieving text:
* **Fetch Content:** `GET /api/v1/content/bible.nt.john.17.3`
* **Explore Hierarchy:** `GET /api/v1/hierarchy/bible.nt.john`
* **Resolve Shorthand:** `GET /api/v1/resolve/bible/Jn 17:3`

## 🗺️ Roadmap
Track our active development and upcoming technical decisions in the [Active Roadmap](docs/decisions/archive/v0.1.2/todo.md).