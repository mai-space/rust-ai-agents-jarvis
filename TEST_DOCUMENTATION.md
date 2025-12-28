# Test Documentation for Jarvis

This document provides comprehensive documentation for the test suite in the Jarvis project. It covers test organization, patterns, conventions, and how to write effective tests for this codebase.

## Table of Contents

1. [Test Organization](#test-organization)
2. [Test Categories](#test-categories)
3. [Running Tests](#running-tests)
4. [Writing Tests](#writing-tests)
5. [Test Patterns](#test-patterns)
6. [Mocking and Test Doubles](#mocking-and-test-doubles)
7. [Best Practices](#best-practices)

## Test Organization

The test suite is organized into two main locations:

### Unit Tests (`src/**/*.rs`)
- Located inline within source files using `#[cfg(test)]` modules
- Test individual functions and methods in isolation
- Examples: `src/tools/fs.rs`, `src/metrics.rs`, `src/config.rs`

### Integration Tests (`tests/**/*_test.rs`)
- Located in the `tests/` directory
- Test interactions between multiple components
- Test end-to-end workflows
- Each test file corresponds to a specific feature or module

## Test Categories

### 1. Agent Tests
**Location:** `tests/integration_test.rs`, `tests/phase*_test.rs`

Tests for the agent system including:
- Agent handoffs and workflow
- Agent parsing and output formatting
- Agent tool calling
- Human-in-the-loop (HITL) escalation
- Loop detection and prevention

**Key Test:**
```rust
#[tokio::test]
async fn test_manager_flow() {
    // Tests complete agent workflow from ProductOwner to Librarian
}
```

### 2. Tool Tests
**Location:** `src/tools/*.rs` (unit tests)

Tests for individual tools used by agents:
- File system operations (`list_files`, `read_file`, `write_file`, `apply_patch`)
- Git operations (`read_diff`, `git_commit`, `git_checkout`)
- Shell operations (`run_tests`, `static_analysis`)
- Analysis tools (`analyze_dependencies`, `find_code_markers`)
- Cache tools (`cache_project_structure`, `get_cached_structure`)
- Memory tools (`store_preference`)

**Example:**
```rust
#[tokio::test]
async fn test_list_files() {
    // Tests that list_files tool returns correct directory contents
}
```

### 3. Provider Tests
**Location:** `src/providers/ollama.rs` (unit tests)

Tests for LLM and database providers:
- Ollama provider initialization
- Vector database operations (with project scoping)
- Persistence provider operations

### 4. Project Context Tests
**Location:** `src/project_context.rs` (unit tests)

Tests for project detection and context management:
- Project metadata extraction
- Structure caching
- Cache validity checks

**Key Tests:**
```rust
#[test]
fn test_project_metadata_from_path() {
    // Tests project type detection (Rust, Node.js, Python, etc.)
}

#[test]
fn test_structure_cache_validity() {
    // Tests that cached structures are invalidated when files change
}
```

### 5. Metrics Tests
**Location:** `src/metrics.rs` (unit tests)

Tests for agent performance tracking:
- Handoff tracking
- Success rate calculation
- Chain length analysis
- Loop detection
- JSON serialization of metrics

**Example:**
```rust
#[test]
fn test_handoff_metrics_basic() {
    // Tests basic handoff tracking and metrics calculation
}
```

### 6. Config Tests
**Location:** `src/config.rs` (unit tests)

Tests for configuration management:
- Configuration file paths
- Serialization/deserialization
- Default values

### 7. Memory Tests
**Location:** `src/tools/memory.rs` (unit tests), `tests/phase7_test.rs`

Tests for long-term memory and RAG:
- Preference storage
- Vector database integration
- Context retrieval

### 8. GUI Tests
**Location:** `tests/gui_test.rs`

Tests for web interface:
- HTTP endpoints
- Chat functionality
- File upload
- Session management
- Settings management

**Key Pattern:**
```rust
#[tokio::test]
async fn test_gui_index_route() {
    let app = create_gui_app(manager, config);
    let response = app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

### 9. Session Tests
**Location:** `tests/session_id_test.rs`

Tests for session management:
- Session ID generation
- Session persistence
- Session resumption

### 10. Context Files Tests
**Location:** `tests/context_files_test.rs`, `tests/context_files_integration_test.rs`

Tests for context file handling:
- Context file parsing
- Context file validation
- Context file integration in agent workflows

### 11. Event System Tests
**Location:** `tests/events_test.rs`

Tests for real-time event broadcasting:
- Event broadcasting to multiple subscribers
- Event type coverage (AgentStarted, ToolCall, Handoff, etc.)
- TaskSummary tracking and markdown generation
- Subscriber management and cleanup

**Key Tests:**
```rust
#[tokio::test]
async fn test_event_broadcaster_multiple_subscribers() {
    // Tests that events are broadcast to all active subscribers
}

#[test]
fn test_task_summary_file_operations() {
    // Tests file operation tracking in TaskSummary
}
```

### 12. MCP (Model Context Protocol) Tests
**Location:** `tests/mcp_types_test.rs`

Tests for MCP protocol implementation:
- JSON-RPC request/response serialization
- MCP tool definitions
- Tool execution results
- Error handling

**Key Tests:**
```rust
#[test]
fn test_mcp_complete_interaction() {
    // Tests complete MCP request-response cycle
}
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test File
```bash
cargo test --test integration_test
cargo test --test events_test
cargo test --test mcp_types_test
```

### Run Unit Tests Only
```bash
cargo test --lib
```

### Run Integration Tests Only
```bash
cargo test --test '*'
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Parallel
```bash
cargo test -- --test-threads=4
```

### Run a Specific Test
```bash
cargo test test_manager_flow
```

## Writing Tests

### Test Function Structure

Every test should follow this structure:

```rust
/// Brief description of what is being tested
/// 
/// Longer explanation of:
/// - What the test validates
/// - Why it's important
/// - Any edge cases covered
#[tokio::test]  // or #[test] for synchronous tests
async fn test_descriptive_name() {
    // Arrange: Set up test data and dependencies
    let input = setup_test_data();
    
    // Act: Execute the code under test
    let result = function_under_test(input).await;
    
    // Assert: Verify the results
    assert_eq!(result.status, ExpectedStatus);
    assert!(result.data.contains("expected value"));
}
```

### Naming Conventions

- Test functions: `test_<feature>_<scenario>`
  - Examples: `test_manager_flow`, `test_handoff_metrics_basic`
- Helper functions: `<action>_<object>`
  - Examples: `register_all_agents`, `setup_mock_llm`
- Test files: `<module>_test.rs` or `<feature>_test.rs`
  - Examples: `events_test.rs`, `integration_test.rs`

### Documentation Guidelines

Every test should have:

1. **Summary line**: One-line description of what is tested
2. **Detailed description**: Multi-line explanation covering:
   - What aspect of functionality is validated
   - Why this test is important
   - Edge cases or scenarios covered
3. **Inline comments**: For complex setup or assertions

Example:
```rust
/// Test that TaskSummary correctly tracks file operations
/// 
/// This ensures that:
/// - Created files are tracked without duplicates
/// - Modified files are tracked without duplicates
/// - Deleted files are tracked without duplicates
/// - Read operations are ignored (not tracked in summary)
#[test]
fn test_task_summary_file_operations() {
    // Test implementation...
}
```

## Test Patterns

### Pattern 1: Mock LLM Provider

For testing agents without actual LLM calls:

```rust
use jarvis::providers::mock::MockLlm;

#[tokio::test]
async fn test_with_mock_llm() {
    let llm = Arc::new(MockLlm);
    let agent = Arc::new(ProductOwner::new(llm, vec![]));
    // ... test agent behavior
}
```

### Pattern 2: Custom Mock LLM

For testing specific agent responses:

```rust
struct CustomLlm;

#[async_trait::async_trait]
impl LlmProvider for CustomLlm {
    async fn generate(&self, prompt: &str) -> Result<String> {
        if prompt.contains("specific condition") {
            Ok("HANDOFF NextAgent \"reason\" \"context\"".to_string())
        } else {
            Ok("SUCCESS result".to_string())
        }
    }
    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> { 
        Ok(vec![]) 
    }
}
```

### Pattern 3: Temporary Test Files

For testing file operations:

```rust
use tempfile::TempDir;

#[tokio::test]
async fn test_file_operation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Perform file operations
    std::fs::write(&test_file, "content").unwrap();
    
    // Assertions...
    
    // temp_dir is automatically cleaned up when dropped
}
```

### Pattern 4: Register All Agents

For integration tests requiring complete agent chain:

```rust
fn register_all_agents(manager: &mut Manager, llm: Arc<MockLlm>) {
    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    let re = Arc::new(RequirementsEngineer::new(llm.clone(), vec![]));
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
    // ... register all agents
    
    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    // ... register all
}
```

### Pattern 5: Event Broadcasting Tests

For testing real-time event system:

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_event_broadcasting() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver = broadcaster.subscribe().await;
    
    broadcaster.agent_started("TestAgent".to_string()).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed");
    
    match event {
        AgentEvent::AgentStarted { agent_name, .. } => {
            assert_eq!(agent_name, "TestAgent");
        }
        _ => panic!("Wrong event type"),
    }
}
```

## Mocking and Test Doubles

### Available Mocks

1. **MockLlm** (`src/providers/mock.rs`)
   - Returns "Default mock response" or "SUCCESS ..." / "HANDOFF ..." based on prompt
   - Use for testing agent flow without real LLM

2. **MockVectorDb** (defined in tests)
   - Stores embeddings in memory
   - Returns stored results on search

3. **MockPersistence** (defined in tests)
   - Stores session state in memory
   - Returns state on load

### Creating Custom Mocks

When creating mocks:
- Implement the full trait interface
- Keep logic simple and predictable
- Document the mock's behavior
- Use Arc/Mutex for shared state if needed

Example:
```rust
struct MockTool;

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str { "mock_tool" }
    fn description(&self) -> &str { "A mock tool for testing" }
    async fn run(&self, input: Value) -> Result<Value> {
        Ok(json!({"status": "ok"}))
    }
}
```

## Best Practices

### 1. Test Independence
- Each test should be completely independent
- Don't rely on test execution order
- Clean up resources (use tempfile for files, drop for cleanup)

### 2. Clear Assertions
```rust
// Good: Clear what is expected
assert_eq!(result.status, "success");
assert!(result.data.len() > 0, "Expected data to be non-empty");

// Avoid: Unclear failures
assert!(result.is_ok());
```

### 3. Test One Thing
- Each test should verify one specific behavior
- Split complex scenarios into multiple tests
- Use descriptive test names

### 4. Use Helper Functions
```rust
// Reusable helper for common setup
fn create_test_agent(llm: Arc<dyn LlmProvider>) -> Arc<dyn Agent> {
    Arc::new(ProductOwner::new(llm, vec![]))
}
```

### 5. Document Edge Cases
```rust
/// Test that agent handles empty input gracefully
/// 
/// Edge case: When no task is provided, the agent should
/// return an error rather than attempting to process.
#[tokio::test]
async fn test_agent_empty_input() {
    // ...
}
```

### 6. Async Test Guidelines
- Use `#[tokio::test]` for async tests
- Use `timeout()` to prevent hanging tests
- Clean up async resources properly

### 7. Test Coverage Priority
Focus on:
1. Core business logic (agent workflows, handoffs)
2. Error handling paths
3. Edge cases (empty inputs, boundary conditions)
4. Integration points between modules

### 8. Avoid Over-Mocking
- Only mock external dependencies (LLM, database)
- Test real code paths when possible
- Integration tests should use minimal mocking

## Important Notes

### Agent Testing Requirements

⚠️ **CRITICAL**: When testing agent workflows, you MUST register the complete agent chain.

The `MockLlm` expects all agents in the handoff chain to be registered. If testing `ProductOwner`, you must also register `RequirementsEngineer`, `SeniorDeveloper`, and all subsequent agents that may be involved.

**Example:**
```rust
let llm = Arc::new(MockLlm);
let mut manager = Manager::new(3);

// Register ALL agents in the chain
manager.register_agent("ProductOwner".to_string(), po);
manager.register_agent("RequirementsEngineer".to_string(), re);
manager.register_agent("SeniorDeveloper".to_string(), dev);
manager.register_agent("AccessibilityExpert".to_string(), accessibility);
manager.register_agent("SEOExpert".to_string(), seo);
manager.register_agent("SecurityExpert".to_string(), security);
manager.register_agent("QATester".to_string(), qa);
manager.register_agent("Librarian".to_string(), lib);
```

See `tests/gui_test.rs` for the `register_all_agents` helper function.

### Test Performance

- Unit tests should run in milliseconds
- Integration tests should run in < 1 second each
- Use `#[ignore]` for slow tests that require external resources

### Continuous Integration

Tests run automatically on:
- Push to main branch
- Pull requests to main
- Multiple platforms: Ubuntu, macOS, Windows

CI checks:
- `cargo build --verbose`
- `cargo test --verbose`
- `cargo clippy -- -D warnings`

## Test Statistics

Current test coverage (as of last update):

| Category | Tests | Status |
|----------|-------|--------|
| Unit Tests | 32 | ✅ Passing |
| Agent Tests | 4 | ✅ Passing |
| Phase Tests | 11 | ✅ Passing |
| GUI Tests | 13 | ✅ Passing |
| Context Files Tests | 6 | ✅ Passing |
| Event System Tests | 10 | ✅ Passing |
| MCP Types Tests | 12 | ✅ Passing |
| **Total** | **88** | ✅ **All Passing** |

## Contributing

When adding new features:
1. Write tests first (TDD)
2. Follow existing test patterns
3. Document your tests thoroughly
4. Ensure all tests pass: `cargo test`
5. Run clippy: `cargo clippy -- -D warnings`

## Questions?

For questions about:
- Test infrastructure: See `tests/integration_test.rs`
- Mocking patterns: See `src/providers/mock.rs`
- Helper functions: See `tests/gui_test.rs`
- Event testing: See `tests/events_test.rs`
- MCP testing: See `tests/mcp_types_test.rs`
