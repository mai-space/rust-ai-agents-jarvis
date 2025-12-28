/// Tests for MCP (Model Context Protocol) types
/// 
/// The MCP types module defines the data structures for communicating
/// with external MCP-compliant servers. These tests verify:
/// - JSON serialization/deserialization works correctly
/// - Type structures match the MCP protocol specification
/// - Error handling structures are properly defined

use jarvis::mcp::types::*;
use serde_json::json;

/// Test JsonRpcRequest serialization
/// 
/// Verifies that JsonRpcRequest can be serialized to JSON
/// matching the JSON-RPC 2.0 specification.
#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: json!({}),
        id: json!(1),
    };
    
    let serialized = serde_json::to_value(&request).expect("Failed to serialize");
    
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert_eq!(serialized["method"], "tools/list");
    assert_eq!(serialized["id"], 1);
    assert!(serialized.get("params").is_some());
}

/// Test JsonRpcRequest deserialization
/// 
/// Ensures that JSON-RPC requests can be properly parsed
/// from JSON strings.
#[test]
fn test_jsonrpc_request_deserialization() {
    let json_str = r#"{
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {"name": "test_tool", "arguments": {}},
        "id": 42
    }"#;
    
    let request: JsonRpcRequest = serde_json::from_str(json_str)
        .expect("Failed to deserialize");
    
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "tools/call");
    assert_eq!(request.id, 42);
    assert!(request.params.is_object());
}

/// Test JsonRpcResponse with successful result
/// 
/// Verifies that successful JSON-RPC responses are properly
/// represented with a result field and no error.
#[test]
fn test_jsonrpc_response_success() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({"status": "ok"})),
        error: None,
        id: json!(1),
    };
    
    let serialized = serde_json::to_value(&response).expect("Failed to serialize");
    
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert!(serialized["result"].is_object());
    assert!(serialized["error"].is_null());
}

/// Test JsonRpcResponse with error
/// 
/// Ensures that error responses contain the proper error structure
/// and no result field.
#[test]
fn test_jsonrpc_response_error() {
    let error = JsonRpcError {
        code: -32601,
        message: "Method not found".to_string(),
        data: Some(json!({"method": "unknown_method"})),
    };
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(error),
        id: json!(1),
    };
    
    let serialized = serde_json::to_value(&response).expect("Failed to serialize");
    
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert!(serialized["result"].is_null());
    assert_eq!(serialized["error"]["code"], -32601);
    assert_eq!(serialized["error"]["message"], "Method not found");
}

/// Test McpTool structure
/// 
/// Verifies that MCP tool definitions can be properly
/// serialized and deserialized, including tool metadata
/// and input schema.
#[test]
fn test_mcp_tool_structure() {
    let tool = McpTool {
        name: "search".to_string(),
        description: "Search the web".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"]
        }),
    };
    
    let serialized = serde_json::to_value(&tool).expect("Failed to serialize");
    
    assert_eq!(serialized["name"], "search");
    assert_eq!(serialized["description"], "Search the web");
    assert!(serialized["input_schema"].is_object());
    assert_eq!(serialized["input_schema"]["type"], "object");
}

/// Test McpListToolsResult
/// 
/// Ensures that the response from listing MCP tools
/// can contain multiple tool definitions.
#[test]
fn test_mcp_list_tools_result() {
    let tools = vec![
        McpTool {
            name: "tool1".to_string(),
            description: "First tool".to_string(),
            input_schema: json!({}),
        },
        McpTool {
            name: "tool2".to_string(),
            description: "Second tool".to_string(),
            input_schema: json!({}),
        },
    ];
    
    let result = McpListToolsResult { tools };
    
    let serialized = serde_json::to_value(&result).expect("Failed to serialize");
    
    assert!(serialized["tools"].is_array());
    assert_eq!(serialized["tools"].as_array().unwrap().len(), 2);
    assert_eq!(serialized["tools"][0]["name"], "tool1");
    assert_eq!(serialized["tools"][1]["name"], "tool2");
}

/// Test McpCallToolResult with successful execution
/// 
/// Verifies that tool execution results contain content
/// and properly track success/error status.
#[test]
fn test_mcp_call_tool_result_success() {
    let content = McpContent {
        type_: "text".to_string(),
        text: Some("Tool executed successfully".to_string()),
    };
    
    let result = McpCallToolResult {
        content: vec![content],
        is_error: false,
    };
    
    let serialized = serde_json::to_value(&result).expect("Failed to serialize");
    
    assert!(serialized["content"].is_array());
    assert_eq!(serialized["content"].as_array().unwrap().len(), 1);
    assert_eq!(serialized["content"][0]["type"], "text");
    assert_eq!(serialized["is_error"], false);
}

/// Test McpCallToolResult with error
/// 
/// Ensures that tool execution errors are properly
/// represented with the is_error flag set.
#[test]
fn test_mcp_call_tool_result_error() {
    let content = McpContent {
        type_: "text".to_string(),
        text: Some("Tool execution failed".to_string()),
    };
    
    let result = McpCallToolResult {
        content: vec![content],
        is_error: true,
    };
    
    let serialized = serde_json::to_value(&result).expect("Failed to serialize");
    
    assert_eq!(serialized["is_error"], true);
    assert_eq!(serialized["content"][0]["text"], "Tool execution failed");
}

/// Test McpContent with different types
/// 
/// Verifies that MCP content can represent different
/// content types (text, image, etc.) as per the protocol.
#[test]
fn test_mcp_content_types() {
    // Text content
    let text_content = McpContent {
        type_: "text".to_string(),
        text: Some("Hello, world!".to_string()),
    };
    
    let serialized = serde_json::to_value(&text_content).expect("Failed to serialize");
    assert_eq!(serialized["type"], "text");
    assert_eq!(serialized["text"], "Hello, world!");
    
    // Content without text (e.g., image)
    let image_content = McpContent {
        type_: "image".to_string(),
        text: None,
    };
    
    let serialized = serde_json::to_value(&image_content).expect("Failed to serialize");
    assert_eq!(serialized["type"], "image");
    assert!(serialized["text"].is_null());
}

/// Test deserialization of complete MCP interaction
/// 
/// This test simulates a complete request-response cycle
/// to ensure all types work together correctly.
#[test]
fn test_mcp_complete_interaction() {
    // Simulate listing tools
    let list_tools_response = json!({
        "jsonrpc": "2.0",
        "result": {
            "tools": [
                {
                    "name": "brave_search",
                    "description": "Search the web using Brave",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "query": {"type": "string"}
                        }
                    }
                }
            ]
        },
        "id": 1
    });
    
    let response: JsonRpcResponse = serde_json::from_value(list_tools_response)
        .expect("Failed to deserialize response");
    
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    
    let result_value = response.result.unwrap();
    let list_result: McpListToolsResult = serde_json::from_value(result_value)
        .expect("Failed to deserialize tools list");
    
    assert_eq!(list_result.tools.len(), 1);
    assert_eq!(list_result.tools[0].name, "brave_search");
    
    // Simulate calling a tool
    let call_tool_response = json!({
        "jsonrpc": "2.0",
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "Search results: ..."
                }
            ],
            "is_error": false
        },
        "id": 2
    });
    
    let response: JsonRpcResponse = serde_json::from_value(call_tool_response)
        .expect("Failed to deserialize response");
    
    let result_value = response.result.unwrap();
    let call_result: McpCallToolResult = serde_json::from_value(result_value)
        .expect("Failed to deserialize call result");
    
    assert!(!call_result.is_error);
    assert_eq!(call_result.content.len(), 1);
    assert_eq!(call_result.content[0].text.as_ref().unwrap(), "Search results: ...");
}

/// Test default behavior for is_error field
/// 
/// The is_error field should default to false when not present
/// in the JSON response.
#[test]
fn test_mcp_call_tool_result_default_is_error() {
    let json_str = r#"{
        "content": [
            {"type": "text", "text": "Result"}
        ]
    }"#;
    
    let result: McpCallToolResult = serde_json::from_str(json_str)
        .expect("Failed to deserialize");
    
    assert!(!result.is_error, "is_error should default to false");
}

/// Test JsonRpcError with optional data field
/// 
/// Verifies that the data field in errors is optional
/// and can be omitted.
#[test]
fn test_jsonrpc_error_optional_data() {
    let error_with_data = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: Some(json!({"detail": "Missing required field"})),
    };
    
    let serialized = serde_json::to_value(&error_with_data).expect("Failed to serialize");
    assert!(serialized["data"].is_object());
    
    let error_without_data = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };
    
    let serialized = serde_json::to_value(&error_without_data).expect("Failed to serialize");
    assert!(serialized["data"].is_null());
}
