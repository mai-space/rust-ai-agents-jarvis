# Jarvis: Rust-Based AI Agent Squad

Jarvis is a high-performance, secure AI agent framework written in Rust, designed to orchestrate a squad of specialized agents to handle complex coding tasks autonomously.

## 1. Prerequisites
- **Rust & Cargo:** Latest stable version.
- **Ollama:** Running locally with required models (e.g., `llama3`, `codellama`, or `mistral`).
- **PostgreSQL:** With the `pgvector` extension installed.
- **Git:** For codebase management.

## 2. Technical Stack
- **Core Runtime:** `tokio` (Asynchronous execution).
- **Communication:** `ollama-rs` (LLM interaction).
- **Database:** `sqlx` (Asynchronous SQL toolkit) for PostgreSQL.
- **CLI:** `clap` (Command-line argument parsing).
- **Logging & Tracing:** `tracing` & `tracing-subscriber`.
- **Error Handling:** `anyhow` & `thiserror`.
- **Serialization:** `serde` & `serde_json`.

## 3. Architecture: The "Manager Hub" Model
To ensure stability and prevent infinite loops, Jarvis uses a **Central Manager (The Hub)** to orchestrate specialized agents (**The Spokes**).

### Core Components
- **The Manager:** The central brain that receives the user request, maintains the global state, and decides which agent to call next based on the workflow or current task status.
- **Provider Traits (Modularity):** To support multiple LLM backends (Ollama, OpenAI, Anthropic) and Vector Databases, the system uses abstraction traits:
  - `LlmProvider` trait for text generation and embeddings.
  - `VectorDbProvider` trait for storage and retrieval.
- **Agent Trait:** Every agent implements a standard interface:
  - `identity()` -> System prompt and role.
  - `capabilities()` -> List of tools the agent can use.
  - `process(context)` -> Execution logic.
- **Toolbox:** A secure, modular system for agents to interact with the environment (File IO, Git, Shell).

## 4. Agent Squad Blueprint

| Phase | Role | Identity / Responsibility | Key Tools | Handoff Target |
| :--- | :--- | :--- | :--- | :--- |
| **1. Planning** | **Product Owner (PO)** | Orchestrates the feature. Scans codebase structure. | `list_files`, `read_structure` | Requirements Engineer |
| | **Requirements Engineer** | Translates PO context into a technical step-by-step plan. | `None` (Pure Logic) | Senior Developer |
| **2. Implementation** | **Senior Developer** | Expert implementation. Writes clean, modular code. | `write_file`, `apply_patch` | Security Expert |
| **3. Review** | **Security Expert** | Scans for SQLi, XSS, and weak dependencies. | `static_analysis` | QA Tester (Pass) / Developer (Fail) |
| | **Accessibility Expert** | *Optional (Frontend)*: Checks ARIA, contrast, and semantics. | `read_diff` | SEO Expert |
| | **SEO Expert** | *Optional (Frontend)*: Ensures meta tags and SSR compatibility. | `read_diff` | Security Expert |
| **4. Validation** | **QA Tester** | Writes and runs tests. Validates feature completeness. | `run_tests`, `write_test` | Librarian (Pass) / Developer (Fail) |
| | **Librarian** | Finalizes task. Updates documentation and KDocs. | `write_file`, `read_file` | **Complete** |

## 5. Reliability & Loop Management
To prevent agents from getting stuck in "correction loops" (e.g., QA finding the same bug repeatedly):
- **Retry Budget:** Each transition has a `MAX_RETRIES` (default: 3).
- **Context Injection:** When a task is sent back (e.g., QA -> Dev), the failure logs and previous attempts are explicitly added to the next prompt.
- **Human Escalation:** If `MAX_RETRIES` is reached, the Manager pauses execution and requests human intervention via the CLI.

## 6. Development Roadmap

### Phase 1: Foundation (Infrastructure)
- [ ] Initialize Rust workspace and core crates.
- [ ] Implement Ollama client and Postgres/pgvector integration.
- [ ] Design the base `Agent` and `Tool` traits.

### Phase 2: Core Orchestration (The Hub)
- [ ] Build the `Manager` to handle state transitions.
- [ ] Implement basic File System tools (Read/List).
- [ ] Create the CLI wrapper for initial user input.

### Phase 3: Agent Realization (The Spokes)
- [ ] Implement PO and Requirements Engineer logic.
- [ ] Build the Senior Developer with code-writing capabilities.
- [ ] Implement the QA Tester with test-running capabilities.

### Phase 4: Polish & Safety
- [ ] Add Security Expert and Librarian.
- [ ] Implement Loop Management and Human-in-the-loop triggers.
- [ ] Comprehensive testing and documentation.