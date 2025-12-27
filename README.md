# Jarvis: Rust-Based AI Agent Squad

Jarvis is a high-performance, autonomous AI agent framework written in Rust. It utilizes a "Manager-Hub" architecture to orchestrate a squad of specialized agents (The Spokes) to solve complex software engineering tasks.

## 🚀 Key Features

- **Autonomous Squad:** A full pipeline of agents: Product Owner, Requirements Engineer, Senior Developer, Accessibility/SEO Experts, Security Expert, QA Tester, and Librarian.
- **Long-Term Memory:** Integrated RAG (Retrieval-Augmented Generation) using PostgreSQL and `pgvector` to remember project patterns and codebases.
- **Session Persistence:** State-based persistence allows you to stop and resume complex tasks using unique session IDs.
- **Autonomous Tooling:** Agents can use Git, File System, and Shell tools to implement features, run tests, and commit changes.
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

We provide installation scripts to help you set up dependencies like Ollama and PostgreSQL with `pgvector` on various platforms.

### Linux
```bash
chmod +x scripts/install_linux.sh
./scripts/install_linux.sh
```

### macOS
```bash
chmod +x scripts/install_macos.sh
./scripts/install_macos.sh
```

### Windows
```powershell
.\scripts\install_windows.ps1
```

### Manual Configuration
1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-repo/jarvis.git
   cd jarvis
   ```

2. **Configure Database:**
   Ensure PostgreSQL is running and you have a database created. Set the `DATABASE_URL` environment variable:
   ```bash
   export DATABASE_URL="postgres://user:password@localhost/jarvis"
   ```

3. **Install Dependencies:**
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
cargo run --package jarvis -- \
  --task "Implement a simple REST API for user registration using Axum" \
  --model "llama3" \
  --database-url $DATABASE_URL
```

### Resuming a Session
If a task was interrupted or you want to continue working on it:

```bash
cargo run --package jarvis -- \
  --task "Continue previous task" \
  --session-id "your-unique-session-id" \
  --database-url $DATABASE_URL
```

## 🔌 Model Context Protocol (MCP) & IDE Integration

Jarvis is designed to be highly extensible and integrated into professional development environments.

### Using External MCP Tools
You can extend Jarvis's capabilities by connecting it to any MCP-compliant server. Create an `mcp_config.json`:
```json
{
  "mcpServers": {
    "everything": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-everything"]
    }
  }
}
```
Run with:
```bash
cargo run --package jarvis -- --task "..." --mcp-config mcp_config.json
```

### JetBrains IDE Integration (ACP)
Jarvis implements the **Agent Client Protocol (ACP)**, allowing it to act as a backend for JetBrains IDEs.
1. Start Jarvis in ACP mode:
   ```bash
   cargo run --package jarvis -- --serve-acp --acp-port 8000
   ```
2. Connect your IDE to `http://localhost:8000` using the Agent Protocol client.

### Exposing Jarvis as an MCP Server
You can also let other AI assistants (like JetBrains AI) use Jarvis's specialized squad as a tool:
```bash
cargo run --package jarvis -- --serve-mcp
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
