/// Tests for Phase 10: Extensibility & IDE Integration (MCP/ACP)
/// 
/// These tests verify the Model Context Protocol (MCP) and Agent Client Protocol (ACP)
/// implementations that allow Jarvis to integrate with IDEs and external tools.
/// 
/// Tests cover:
/// - MCP client-server communication
/// - MCP tool listing and execution
/// - ACP HTTP endpoints for IDE integration
/// - Task creation and step execution via ACP

use jarvis::mcp::{McpClient, McpServer};
use jarvis::orchestration::{Manager, acp};
use jarvis::providers::mock::MockLlm;
use jarvis::agents::planning::ProductOwner;
use serde_json::{json, Value};
use anyhow::Result;
use std::sync::Arc;
use tokio::io::duplex;
use jarvis::tools::Tool;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

/// Mock tool for testing MCP functionality
struct TestTool;
#[async_trait]
impl Tool for TestTool {
    fn name(&self) -> &str { "test_tool" }
    fn description(&self) -> &str { "test tool desc" }
    async fn run(&self, input: Value) -> Result<Value> {
        Ok(json!({ "echo": input }))
    }
}

/// Test MCP client-server integration
/// 
/// Verifies end-to-end MCP communication:
/// - Client can connect to server
/// - Tools can be listed via MCP protocol
/// - Tools can be called with arguments
/// - Results are properly formatted and returned
/// 
/// Uses in-memory duplex stream to avoid network overhead.
#[tokio::test]
async fn test_mcp_client_server_integration() -> Result<()> {
    let (client_io, server_io) = duplex(1024);
    let (client_reader, client_writer) = tokio::io::split(client_io);
    let (server_reader, server_writer) = tokio::io::split(server_io);

    let server = McpServer::new(vec![Arc::new(TestTool)]);
    
    // Run server in background
    tokio::spawn(async move {
        let _ = server.run_on_io(server_reader, server_writer).await;
    });

    // Create client
    // Note: client.initialize() is called inside McpClient::new
    let mut client = McpClient::new(client_reader, client_writer).await?;

    // Test list_tools
    let tools = client.list_tools().await?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "test_tool");

    // Test call_tool
    let result = client.call_tool("test_tool", json!({ "hello": "world" })).await?;
    assert_eq!(result.content.len(), 1);
    let text = result.content[0].text.as_ref().expect("Expected text in content");
    
    // McpServer returns the tool result as a stringified JSON in the content
    let text_val: Value = serde_json::from_str(text)?;
    assert_eq!(text_val["echo"]["hello"], "world");

    Ok(())
}

/// Test ACP (Agent Client Protocol) HTTP endpoints
/// 
/// Verifies IDE integration via HTTP API:
/// - Task creation endpoint accepts input
/// - Task IDs are properly generated
/// - Step execution endpoint runs agent logic
/// - Results contain expected output format
/// 
/// This enables JetBrains and VS Code integration.
#[tokio::test]
async fn test_acp_server_endpoints() -> Result<()> {
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    manager.register_agent("ProductOwner".to_string(), po);
    let manager = Arc::new(manager);

    let app = acp::create_app(manager);

    // 1. Create Task
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/agent/tasks")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "input": "test task" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let task: Value = serde_json::from_slice(&body).unwrap();
    let task_id = task["task_id"].as_str().unwrap();
    assert_eq!(task["input"], "test task");

    // 2. Execute Step
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/agent/tasks/{}/steps", task_id))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let step: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(step["status"], "completed");
    assert!(step["output"].as_str().unwrap().contains("Default mock response"));

    Ok(())
}
