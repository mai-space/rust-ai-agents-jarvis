# Jarvis: Rust-Based AI Agent Squad

Jarvis is a high-performance, autonomous AI agent framework written in Rust. It utilizes a "Manager-Hub" architecture to orchestrate a squad of specialized agents (The Spokes) to solve complex software engineering tasks.

## 🚀 Key Features

- **Autonomous Squad:** A full pipeline of agents: Product Owner, Requirements Engineer, Senior Developer, Accessibility/SEO Experts, Security Expert, QA Tester, and Librarian.
- **Long-Term Memory:** Integrated RAG (Retrieval-Augmented Generation) using PostgreSQL and `pgvector` to remember project patterns and codebases.
- **Project Context Awareness:** Automatic project detection and differentiation with project-scoped memory to avoid confusion between different codebases.
- **Intelligent Loop Prevention:** Advanced handoff validation and loop detection prevents agents from getting stuck in infinite cycles.
- **Smart Caching:** Project structure caching reduces redundant filesystem scans by 10-50x, dramatically improving performance.
- **Session Persistence:** State-based persistence allows you to stop and resume complex tasks using unique session IDs.
- **Autonomous Tooling:** Agents can use Git, File System, Shell, and Code Analysis tools to implement features, run tests, and commit changes.
- **Human-in-the-Loop (HITL):** Built-in escalation mechanism when agents reach retry limits, ensuring safety and control.
- **Model Context Protocol (MCP):** Dynamic tool extension via external MCP servers (e.g., Brave Search, Google Maps).
- **Agent Client Protocol (ACP):** Standardized API for IDE integration (JetBrains, VS Code).
- **Personalized Memory:** Dual-stream RAG that distinguishes between technical project context and individual user preferences.
- **Multi-backend LLM Support:** Primary support for Ollama (local LLMs), extensible via traits.
- **Cross-Platform:** Native support and installers for Linux, macOS, and Windows.

## 🛠 Prerequisites

- **Rust:** Latest stable version.
- **Ollama:** Running locally (recommended models: `llama3`, `codellama`).
- **PostgreSQL:** With the `pgvector` extension installed.
- **Git:** For version control operations.

## 📥 Installation & Setup

### Global Installation (Recommended)
To use Jarvis as a CLI tool in any project, install it globally using cargo:
```bash
# We recommend using --locked to ensure compatibility with Rust 1.75.0+
cargo install --path jarvis --locked
```

After installation, run the interactive setup to configure your environment:
```bash
jarvis setup
```
This will guide you through setting up your Ollama host, model, and database connection. The configuration is stored in your user's standard config directory (e.g., `~/.config/jarvis/config.toml` on Linux).

Once configured, you can run Jarvis from any project directory:
```bash
jarvis --task "Add a new feature to this project"
```

### Scripted Installation
We provide installation scripts to help you set up dependencies like Ollama and PostgreSQL with `pgvector` on various platforms.

#### Linux
```bash
chmod +x scripts/install_linux.sh
./scripts/install_linux.sh
```

#### macOS
```bash
chmod +x scripts/install_macos.sh
./scripts/install_macos.sh
```

#### Windows
```powershell
.\scripts\install_windows.ps1
```

### Manual Development Setup
1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-repo/jarvis.git
   cd jarvis
   ```

2. **Configure Database:**
   Ensure PostgreSQL is running and you have a database created. You can use the `setup` command after installing or set the `DATABASE_URL` environment variable:
   ```bash
   export DATABASE_URL="postgres://user:password@localhost/jarvis"
   ```

3. **Build:**
   ```bash
   cargo build
   ```

## 🖥 Platform Support & Edge Cases

- **Linux:** Native support for Ubuntu/Debian. Other distributions may require manual installation of `pgvector` from source.
- **macOS:** Fully supported via Homebrew. Works on both Intel and Apple Silicon (M1/M2/M3).
- **Windows:** Best experienced using Docker for the database. Native `pgvector` build on Windows can be complex; the `ankane/pgvector` Docker image is the recommended path.

## 📖 Tutorial: Running Your First Task

To start a new task with Jarvis:

```bash
jarvis --task "Implement a simple REST API for user registration using Axum"
```

### Providing Context Files
You can provide specific files as context to help agents work more efficiently. This is especially useful when you want to focus on specific parts of your codebase:

```bash
jarvis --task "Refactor the authentication logic" --context-files src/auth.rs,src/models.rs
```

The context files will be read and included in the agent's prompt, providing immediate access to relevant code without requiring the agent to search for files. Multiple files can be specified using comma-separated paths.

### Resuming a Session
Jarvis automatically generates and displays a session ID when you run a task with persistence enabled (database configured). You can use this session ID to resume work later:

```bash
jarvis --task "Continue previous task" --session-id "abc123..."
```

**Example output:**
```
--- FINAL RESULT ---
Task completed successfully!

--- SESSION INFO ---
Session ID: 550e8400-e29b-41d4-a716-446655440000
To resume this session later, use: --session-id 550e8400-e29b-41d4-a716-446655440000
```

Note: Session persistence requires database configuration. Without a database, sessions are not saved.

## 🔌 Model Context Protocol (MCP) & IDE Integration

Jarvis is designed to be highly extensible and integrated into professional development environments.

### Using External MCP Tools
You can extend Jarvis's capabilities by connecting it to any MCP-compliant server. Create an `mcp_config.json` or configure it via `jarvis setup`.

Run with specific config:
```bash
jarvis --task "..." --mcp-config mcp_config.json
```

### JetBrains IDE Integration (ACP)
Jarvis implements the **Agent Client Protocol (ACP)**, allowing it to act as a backend for JetBrains IDEs.
1. Start Jarvis in ACP mode:
   ```bash
   jarvis --serve-acp --acp-port 8000
   ```
2. Connect your IDE to `http://localhost:8000` using the Agent Protocol client.

### Exposing Jarvis as an MCP Server
You can also let other AI assistants (like JetBrains AI) use Jarvis's specialized squad as a tool:
```bash
jarvis --serve-mcp
```

## 🏗 Extending Jarvis

### Adding a New Tool
Implement the `Tool` trait to add custom capabilities:
```rust
use jarvis::tools::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_custom_tool" }
    fn description(&self) -> &str { "Description of what it does" }
    async fn run(&self, input: Value) -> anyhow::Result<Value> {
        // Your logic here
        Ok(json!({ "status": "success" }))
    }
}
```

### Adding a New Agent
Implement the `Agent` trait and register it in the `Manager`:
```rust
use jarvis::agents::{Agent, AgentContext, AgentOutput};

pub struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    fn identity(&self) -> String { "You are a specialized agent..." .to_string() }
    fn capabilities(&self) -> Vec<Arc<dyn Tool>> { vec![...] }
    async fn process(&self, context: &mut AgentContext) -> anyhow::Result<AgentOutput> {
        // Use jarvis::agents::run_llm_agent or custom logic
    }
}
```

## 🧪 Testing Status

We maintain a suite of integration tests to ensure squad reliability and memory consistency.

| Test Suite | Description | Status |
| :--- | :--- | :--- |
| `integration_test.rs` | Verifies basic agent handoffs and tool calling. | ✅ Passing |
| `phase7_test.rs` | Verifies Vector DB integration, RAG logic, and session persistence. | ✅ Passing |
| `phase8_test.rs` | Verifies Dual-Stream RAG and user preference extraction. | ✅ Passing |

*Note: Unit tests for individual tools are currently in development.*

## 🛣 Roadmap

The project has successfully completed its initial roadmap. See [plan.md](plan.md) for full details on each completed phase.

- [x] Phase 1-7: Core infrastructure, agents, and memory.
- [x] Phase 8: Personalized Memory & Context Awareness.
- [x] Phase 9: Distribution & Cross-Platform Support.
- [x] Phase 10: Extensibility & IDE Integration (MCP/ACP).
