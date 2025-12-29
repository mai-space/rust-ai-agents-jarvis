# Jarvis: Rust-Based AI Agent Squad

Jarvis is a high-performance, autonomous AI agent framework written in Rust. It utilizes a "Manager-Hub" architecture to orchestrate a squad of specialized agents (The Spokes) to solve complex software engineering tasks.

## 🚀 Key Features

### Core Capabilities
- **Autonomous Squad:** A complete pipeline of specialized agents working together:
  - **Product Owner:** Scans codebase, understands project structure
  - **Requirements Engineer:** Translates tasks into technical step-by-step plans
  - **Senior Developer:** Implements features with clean, modular code
  - **Accessibility Expert:** Ensures ARIA labels, contrast, semantic HTML
  - **SEO Expert:** Validates meta tags, SSR compatibility, semantic headers
  - **Security Expert:** Scans for SQL injection, XSS, and security vulnerabilities
  - **QA Tester:** Writes and runs tests, validates feature completeness
  - **Librarian:** Finalizes documentation and stores user preferences

### Intelligence & Memory
- **Long-Term Memory:** Integrated RAG (Retrieval-Augmented Generation) using PostgreSQL and `pgvector` to remember project patterns and codebases
- **Project Context Awareness:** Automatic project detection and differentiation with project-scoped memory to avoid confusion between different codebases
- **Personalized Memory:** Dual-stream RAG that distinguishes between technical project context and individual user preferences
- **Smart Caching:** Project structure caching reduces redundant filesystem scans by 10-50x, dramatically improving performance

### Reliability & Safety
- **Intelligent Loop Prevention:** Advanced handoff validation and loop detection prevents agents from getting stuck in infinite cycles
- **Session Persistence:** State-based persistence allows you to stop and resume complex tasks using unique session IDs
- **Human-in-the-Loop (HITL):** Built-in escalation mechanism when agents reach retry limits, ensuring safety and control
- **Metrics & Monitoring:** Built-in handoff tracking, success rate monitoring, and performance analytics

### Developer Tools
- **Autonomous Tooling:** Agents can use 17+ specialized tools:
  - **File System:** `list_files`, `read_file`, `write_file`, `read_structure`, `apply_patch`, `search_codebase`
  - **Git:** `read_diff`, `git_commit`, `git_checkout`
  - **Shell:** `run_tests`, `static_analysis`
  - **Analysis:** `analyze_dependencies`, `find_code_markers` (TODO/FIXME/HACK)
  - **Memory:** `store_preference` for user preferences
  - **Caching:** `cache_project_structure`, `get_cached_structure`
  - **MCP Tools:** Dynamic tools from external MCP servers

### Integration & Extensibility
- **Model Context Protocol (MCP):** Dynamic tool extension via external MCP servers (e.g., Brave Search, Google Maps)
- **Agent Client Protocol (ACP):** Standardized API for IDE integration (JetBrains, VS Code)
- **Multi-backend LLM Support:** Primary support for Ollama (local LLMs), extensible via traits
- **GUI Mode:** Modern web-based chat interface for intuitive interaction
- **TUI Mode:** Terminal user interface for keyboard-driven, SSH-friendly interaction
- **Cross-Platform:** Native support and installers for Linux, macOS, and Windows

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

### Basic Usage
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

For more details, see [Context Files Documentation](docs/context-files.md).

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

### Using the GUI Mode
Jarvis includes a modern web-based chat interface for a more intuitive experience:

```bash
jarvis --serve-gui
```

Then open your browser to http://localhost:3000

The GUI provides:
- **Intuitive Chat Interface**: Modern, OpenChat-inspired UI
- **File Upload Support**: Drag and drop files or click to attach files for context
- **Session Management**: Automatically managed conversations with session IDs
- **Real-time Updates**: See agent responses as they happen
- **Supported File Types**: Source code (.rs, .js, .ts, .py, .java, .go, etc.), configs (.json, .yaml, .toml), documentation (.md, .html)

You can customize the port:
```bash
jarvis --serve-gui --gui-port 8080
```

See [GUI Mode Documentation](docs/gui-mode.md) for detailed usage instructions.

### Using the TUI Mode
Jarvis also includes a Terminal User Interface (TUI) for a keyboard-driven, terminal-native experience:

```bash
jarvis --serve-tui
```

The TUI provides:
- **Terminal Native**: Works entirely in your terminal, no browser needed
- **Keyboard Shortcuts**: All operations accessible via keyboard
- **File Context Support**: Interactively add files for context
- **Agent Selection**: Choose which agent to use via interactive menu
- **SSH Friendly**: Perfect for remote development over SSH
- **Session Management**: Automatic session tracking

Keyboard shortcuts:
- `i` - Start typing a message
- `Ctrl+D` - Send message
- `f` - Add context file
- `a` - Select agent
- `n` - New chat
- `?` - Help
- `q` - Quit

See [TUI Mode Documentation](docs/tui-mode.md) for detailed usage instructions.

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

## 📊 Metrics & Monitoring

Jarvis includes built-in metrics to track agent performance and identify issues:

- **Handoff Tracking:** Monitor all agent transitions and handoff patterns
- **Success Rate:** Track successful task completions vs. failures
- **Loop Detection:** Identify and prevent circular agent handoffs
- **Chain Length Analysis:** Measure average and maximum agent chain lengths
- **Performance Analytics:** Export metrics as JSON for external analysis

Example metrics output:
```
=== Agent Handoff Metrics ===
Total Handoffs: 5
Successful Completions: 1
Success Rate: 100.0%
Average Chain Length: 6.0
Loop Incidents: 0

Top Handoff Pairs:
  ProductOwner -> RequirementsEngineer: 1 times
  RequirementsEngineer -> SeniorDeveloper: 1 times
```

## 🏗 Extending Jarvis

### Available Built-in Tools

Jarvis comes with 17+ specialized tools organized by category:

**File System Tools:**
- `list_files` - List files in a directory
- `read_file` - Read file contents
- `write_file` - Write content to files
- `read_structure` - Recursively scan project structure
- `apply_patch` - Apply unified diff patches
- `search_codebase` - Search code using embeddings

**Git Tools:**
- `read_diff` - Read Git diffs
- `git_commit` - Commit changes
- `git_checkout` - Checkout branches

**Shell Tools:**
- `run_tests` - Execute test commands
- `static_analysis` - Run static analysis tools

**Analysis Tools:**
- `analyze_dependencies` - Analyze code imports (Rust, JS/TS, Python)
- `find_code_markers` - Find TODO, FIXME, HACK, NOTE markers

**Cache Tools:**
- `cache_project_structure` - Cache project structure for speed
- `get_cached_structure` - Retrieve cached structure (10-50x faster)

**Memory Tools:**
- `store_preference` - Store user preferences

**MCP Tools:**
- Dynamic tools from external MCP servers

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
    fn identity(&self) -> String { 
        "You are a specialized agent...".to_string() 
    }
    fn capabilities(&self) -> Vec<Arc<dyn Tool>> { 
        vec![/* your tools */] 
    }
    async fn process(&self, context: &mut AgentContext) -> anyhow::Result<AgentOutput> {
        // Use jarvis::agents::run_llm_agent or custom logic
        run_llm_agent(self, llm_provider, context).await
    }
}
```

## 🧪 Testing Status

We maintain a comprehensive suite of tests to ensure squad reliability, memory consistency, and tool functionality.

**Test Results:**
- **Total Tests:** 88 (32 unit + 56 integration)
- **Status:** ✅ All Passing (1 test ignored due to known CI issue)
- **Coverage:** Core agents, tools, providers, project context, metrics, GUI integration, events, and MCP

| Test Category | Description | Tests | Status |
| :--- | :--- | :--- | :--- |
| **Agent Tests** | Verifies agent handoffs, parsing, and I/O sanitization | 4 | ✅ Passing |
| **Tool Tests** | Validates all file system, Git, shell, analysis, and cache tools | 15 | ✅ Passing |
| **Provider Tests** | Verifies LLM and Vector DB integration | 1 | ✅ Passing |
| **Project Context** | Validates project detection and structure caching | 3 | ✅ Passing |
| **Metrics** | Tests handoff tracking and performance analytics | 5 | ✅ Passing |
| **Config** | Validates configuration persistence | 2 | ✅ Passing |
| **Memory** | Tests preference storage and retrieval | 2 | ✅ Passing |
| **GUI Integration** | Tests web interface, API endpoints, and file uploads | 13 | ✅ Passing |
| **Integration Tests** | End-to-end agent workflow and session management | 11 | ✅ Passing |
| **Event System** | Tests real-time event broadcasting and TaskSummary | 10 | ✅ Passing |
| **MCP Types** | Tests Model Context Protocol type serialization | 12 | ✅ Passing |
| **Context Files** | Tests context file handling and integration | 6 | ✅ Passing |
| **Phase Tests** | Tests for specific development phases (1 ignored) | 4 | ✅ Passing |

**Note:** One test (`test_acp_server_endpoints`) is marked as ignored due to a known hanging issue in CI. See [TEST_DOCUMENTATION.md](TEST_DOCUMENTATION.md) for details.

Run tests with:
```bash
cargo test
```

For comprehensive test documentation, patterns, and best practices, see [TEST_DOCUMENTATION.md](TEST_DOCUMENTATION.md).

### Important Testing Notes

When writing tests that involve the agent squad:

**⚠️ Register the complete agent chain:** The `MockLlm` expects all agents in the handoff chain to be registered. For example, if testing `ProductOwner`, you must also register `RequirementsEngineer`, `SeniorDeveloper`, and all subsequent agents that may be involved in the handoff sequence.

**Example:**
```rust
let llm = Arc::new(MockLlm);
let mut manager = Manager::new(3);

// Register ALL agents in the chain
let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
let re = Arc::new(RequirementsEngineer::new(llm.clone(), vec![]));
let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
// ... register remaining agents

manager.register_agent("ProductOwner".to_string(), po);
manager.register_agent("RequirementsEngineer".to_string(), re);
// ... register all others
```

This ensures the complete agent workflow can execute without hanging. See [TEST_DOCUMENTATION.md](TEST_DOCUMENTATION.md) for more details and `jarvis/tests/gui_test.rs` for the `register_all_agents` helper function.

## 🛣 Roadmap

The project has successfully completed its initial roadmap through Phase 11. See [plan.md](plan.md) for full details on each completed phase.

**Completed Phases:**
- ✅ Phase 1-7: Core infrastructure, agents, and memory
- ✅ Phase 8: Personalized Memory & Context Awareness
- ✅ Phase 9: Distribution & Cross-Platform Support
- ✅ Phase 10: Extensibility & IDE Integration (MCP/ACP)
- ✅ Phase 11: CLI Empowerment & Global Usage

**Key Achievements:**
- 8 specialized agents working in harmony
- 17+ autonomous tools for code manipulation
- Project-scoped vector database for context isolation
- Smart caching system for 10-50x performance improvement
- Loop detection and prevention mechanisms
- GUI mode for intuitive interaction
- MCP/ACP integration for extensibility
- Cross-platform support (Linux, macOS, Windows)
- Comprehensive testing suite with 32 passing tests

For detailed implementation notes, see [SUMMARY.md](SUMMARY.md) and [IMPROVEMENTS.md](IMPROVEMENTS.md).
