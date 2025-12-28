use axum::http::StatusCode;
use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;
use jarvis::orchestration::gui::create_gui_app;
use jarvis::config::Config;
use jarvis::orchestration::Manager;
use jarvis::providers::mock::MockLlm;
use jarvis::agents::planning::{ProductOwner, RequirementsEngineer};
use jarvis::agents::development::SeniorDeveloper;
use jarvis::agents::refinement::{AccessibilityExpert, SEOExpert};
use jarvis::agents::validation::QATester;
use jarvis::agents::security::SecurityExpert;
use jarvis::agents::documentation::Librarian;
use std::sync::Arc;

// Helper function to register all agents needed for tests
fn register_all_agents(manager: &mut Manager, llm: Arc<MockLlm>) {
    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    let re = Arc::new(RequirementsEngineer::new(llm.clone(), vec![]));
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
    let accessibility = Arc::new(AccessibilityExpert::new(llm.clone(), vec![]));
    let seo = Arc::new(SEOExpert::new(llm.clone(), vec![]));
    let security = Arc::new(SecurityExpert::new(llm.clone(), vec![]));
    let qa = Arc::new(QATester::new(llm.clone(), vec![]));
    let lib = Arc::new(Librarian::new(llm.clone(), vec![]));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("AccessibilityExpert".to_string(), accessibility);
    manager.register_agent("SEOExpert".to_string(), seo);
    manager.register_agent("SecurityExpert".to_string(), security);
    manager.register_agent("QATester".to_string(), qa);
    manager.register_agent("Librarian".to_string(), lib);
}

#[tokio::test]
async fn test_gui_index_route() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    assert!(body_str.contains("Jarvis AI Agent Squad"));
    assert!(body_str.contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn test_gui_settings_get() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    let response = app
        .oneshot(Request::builder().uri("/api/settings").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let settings: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(settings.get("ollama_host").is_some());
    assert!(settings.get("ollama_port").is_some());
    assert!(settings.get("model").is_some());
}

#[tokio::test]
async fn test_gui_chat_endpoint() {
    let mut config = Config::default();
    config.ollama_host = "localhost".to_string();
    config.ollama_port = 11434;
    config.model = "llama3".to_string();
    
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    register_all_agents(&mut manager, llm);
    
    let manager = Arc::new(manager);
    
    let app = create_gui_app(manager, config);
    
    let chat_request = serde_json::json!({
        "session_id": "test-session-123",
        "message": "Hello, create a simple test",
        "context_files": []
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&chat_request).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let chat_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(chat_response.get("session_id").is_some());
    assert!(chat_response.get("response").is_some());
    assert_eq!(chat_response["session_id"], "test-session-123");
}

#[tokio::test]
async fn test_gui_session_retrieval_empty() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/session/nonexistent-session")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let session_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Should return empty messages array for non-existent session
    let messages = session_data["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 0);
}

#[tokio::test]
async fn test_gui_settings_update() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    let new_settings = serde_json::json!({
        "ollama_host": "newhost",
        "ollama_port": 12345,
        "model": "llama3.2",
        "database_url": null,
        "model_config": {
            "planning_model": "planning-model",
            "analysis_model": "analysis-model",
            "coding_model": "coding-model",
            "writing_model": "writing-model"
        }
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&new_settings).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the response returns the updated config
    assert_eq!(response_data["ollama_host"], "newhost");
    assert_eq!(response_data["ollama_port"], 12345);
    assert_eq!(response_data["model"], "llama3.2");
}

#[tokio::test]
async fn test_gui_multiple_sessions() {
    let config = Config::default();
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    register_all_agents(&mut manager, llm);
    let manager = Arc::new(manager);
    
    let app = create_gui_app(manager, config);
    
    // Create first session
    let chat_request1 = serde_json::json!({
        "session_id": "session-1",
        "message": "First message",
        "context_files": []
    });
    
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&chat_request1).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response1.status(), StatusCode::OK);
    
    // Create second session
    let chat_request2 = serde_json::json!({
        "session_id": "session-2",
        "message": "Second message",
        "context_files": []
    });
    
    let response2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&chat_request2).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response2.status(), StatusCode::OK);
    
    // Retrieve first session
    let response3 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/session/session-1")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response3.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response3.into_body(), usize::MAX).await.unwrap();
    let session_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    let messages = session_data["messages"].as_array().unwrap();
    // Should have at least user message
    assert!(messages.len() >= 1);
}

#[tokio::test]
async fn test_gui_agent_selection() {
    let config = Config::default();
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    
    // Register all agents
    register_all_agents(&mut manager, llm);
    
    let manager = Arc::new(manager);
    let app = create_gui_app(manager, config);
    
    // Test with specific agent selection
    let chat_request = serde_json::json!({
        "session_id": "test-session-agent",
        "message": "Test message",
        "agent": "SeniorDeveloper",
        "context_files": []
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&chat_request).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let chat_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(chat_response.get("session_id").is_some());
    assert!(chat_response.get("response").is_some());
}

#[tokio::test]
async fn test_gui_chat_stream_endpoint() {
    let config = Config::default();
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    register_all_agents(&mut manager, llm);
    let manager = Arc::new(manager);
    
    let app = create_gui_app(manager, config);
    
    let chat_request = serde_json::json!({
        "session_id": "test-stream-session",
        "message": "Test streaming",
        "context_files": []
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat/stream")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&chat_request).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check that response has SSE content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    let content_type_str = content_type.unwrap().to_str().unwrap();
    assert!(content_type_str.contains("text/event-stream"));
}

#[tokio::test]
async fn test_gui_default_agent() {
    let config = Config::default();
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    register_all_agents(&mut manager, llm);
    let manager = Arc::new(manager);
    
    let app = create_gui_app(manager, config);
    
    // Test without agent selection (should default to ProductOwner)
    let chat_request = serde_json::json!({
        "session_id": "test-default-agent",
        "message": "Test default agent",
        "context_files": []
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&chat_request).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let chat_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(chat_response["session_id"], "test-default-agent");
    assert!(chat_response.get("response").is_some());
}

#[tokio::test]
async fn test_gui_file_upload_with_preview() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    // Create multipart form data
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = format!(
        "------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\
         \r\n\
         Hello, this is test content!\r\n\
         ------WebKitFormBoundary7MA4YWxkTrZu0gW--\r\n"
    );
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let files: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"], "test.txt");
    assert_eq!(files[0]["content"], "Hello, this is test content!");
}

#[tokio::test]
async fn test_gui_task_status_no_task() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/task/status/test-session-123")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let status: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(status["is_running"], false);
    assert_eq!(status["session_id"], "test-session-123");
}

#[tokio::test]
async fn test_gui_stop_task_not_found() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    // Try to stop a non-existent task
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/task/stop/nonexistent-session")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    // Should return NOT_FOUND
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_gui_html_contains_stop_restart_button() {
    let config = Config::default();
    let manager = Arc::new(Manager::new(3));
    
    let app = create_gui_app(manager, config);
    
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Check for stop/restart button elements
    assert!(body_str.contains("stopRestartButton"));
    assert!(body_str.contains("handleStopRestart"));
    assert!(body_str.contains("isTaskRunning"));
}


