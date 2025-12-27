# Jarvis: Rust-Based AI Agent Squad

Jarvis is a high-performance, autonomous AI agent framework written in Rust. It utilizes a "Manager-Hub" architecture to orchestrate a squad of specialized agents (The Spokes) to solve complex software engineering tasks.

## 🚀 Key Features

- **Autonomous Squad:** A full pipeline of agents: Product Owner, Requirements Engineer, Senior Developer, Accessibility/SEO Experts, Security Expert, QA Tester, and Librarian.
- **Long-Term Memory:** Integrated RAG (Retrieval-Augmented Generation) using PostgreSQL and `pgvector` to remember project patterns and codebases.
- **Session Persistence:** State-based persistence allows you to stop and resume complex tasks using unique session IDs.
- **Autonomous Tooling:** Agents can use Git, File System, and Shell tools to implement features, run tests, and commit changes.
- **Human-in-the-Loop (HITL):** Built-in escalation mechanism when agents reach retry limits, ensuring safety and control.
- **Multi-backend LLM Support:** Primary support for Ollama (local LLMs), extensible via traits.

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

## 🧪 Testing Status

We maintain a suite of integration tests to ensure squad reliability and memory consistency.

| Test Suite | Description | Status |
| :--- | :--- | :--- |
| `integration_test.rs` | Verifies basic agent handoffs and tool calling. | ✅ Passing |
| `phase7_test.rs` | Verifies Vector DB integration, RAG logic, and session persistence. | ✅ Passing |

*Note: Unit tests for individual tools are currently in development.*

## 🛣 Roadmap

See [plan.md](plan.md) for the detailed development roadmap, including upcoming features like personalized user memory and multi-platform distribution.
