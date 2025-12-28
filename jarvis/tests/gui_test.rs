use axum::http::StatusCode;
use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;
use jarvis::orchestration::gui::create_gui_app;
use jarvis::config::Config;
use jarvis::orchestration::Manager;
use jarvis::providers::mock::MockLlm;
use jarvis::agents::planning::ProductOwner;
use std::sync::Arc;

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
    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    manager.register_agent("ProductOwner".to_string(), po);
    
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
    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    manager.register_agent("ProductOwner".to_string(), po);
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

