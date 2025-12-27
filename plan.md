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
| **2. Implementation** | **Senior Developer** | Expert implementation. Writes clean, modular code. | `write_file`, `apply_patch` | Accessibility Expert |
| **3. Review** | **Accessibility Expert** | Checks ARIA, contrast, and semantic tags. Applies fixes. | `read_diff`, `apply_patch` | SEO Expert |
| | **SEO Expert** | Ensures meta tags, SSR compatibility, and headers. Applies fixes. | `read_diff`, `apply_patch` | Security Expert |
| | **Security Expert** | Scans for SQLi, XSS, and weak dependencies. | `static_analysis` | QA Tester (Pass) / Developer (Fail) |
| **4. Validation** | **QA Tester** | Writes and runs tests. Validates feature completeness. | `run_tests`, `write_test` | Librarian (Pass) / Developer (Fail) |
| | **Librarian** | Finalizes task. Updates documentation and KDocs. | `write_file`, `read_file` | **Complete** |

## 5. Reliability & Loop Management
To prevent agents from getting stuck in "correction loops" (e.g., QA finding the same bug repeatedly):
- **Retry Budget:** Each transition has a `MAX_RETRIES` (default: 3).
- **Context Injection:** When a task is sent back (e.g., QA -> Dev), the failure logs and previous attempts are explicitly added to the next prompt.
- **Human Escalation:** If `MAX_RETRIES` is reached, the Manager pauses execution and requests human intervention via the CLI.

## 6. Development Roadmap

### Phase 1: Foundation (Infrastructure)
- [x] Initialize Rust workspace and core crates.
- [x] Implement Ollama client and Postgres/pgvector integration.
- [x] Design the base `Agent` and `Tool` traits.

### Phase 2: Core Orchestration (The Hub)
- [x] Build the `Manager` to handle state transitions.
- [x] Implement basic File System tools (Read/List).
- [x] Create the CLI wrapper for initial user input.

### Phase 3: Agent Realization (The Spokes)
- [x] Implement PO and Requirements Engineer logic.
- [x] Build the Senior Developer with code-writing capabilities.
- [x] Implement the QA Tester with test-running capabilities.

### Phase 4: Polish & Safety
- [x] Add Security Expert and Librarian.
- [x] Implement Loop Management and Human-in-the-loop triggers.
- [x] Comprehensive testing and documentation.

### Phase 5: Refined Review & Tools
- [x] Implement `AccessibilityExpert` and `SEOExpert` agents.
- [x] Implement `read_diff` and `apply_patch` tools.
- [x] Refine handoff logic to include the full review chain.
- [x] Update integration tests to cover the expanded squad.

### Phase 6: Real Tooling & Autonomous Execution
- [x] Implement `ReadStructureTool` for recursive codebase scanning.
- [x] Implement autonomous tool-calling loop in agents.
- [x] Connect all specialized agents to their respective 'real' tools.
- [x] Verify tool-calling autonomous behavior with integration tests.

### Phase 7: Long-Term Memory & Advanced Tooling
- [ ] Integrate `VectorDbProvider` (Postgres) into `Manager` and `AgentContext`.
- [ ] Implement RAG (Retrieval-Augmented Generation) logic in `run_llm_agent` to automatically pull relevant context from the vector database.
- [ ] Add `GitCommitTool` and `GitCheckoutTool` for autonomous version control management.
- [ ] Implement Global State persistence (saving `AgentContext` to Postgres) to allow resuming tasks after a restart.
- [ ] Enhance `ApplyPatchTool` with better conflict resolution and error reporting.
- [ ] Add a `SearchCodebaseTool` that uses embeddings to find relevant code snippets.