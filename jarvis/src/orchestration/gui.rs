use crate::orchestration::Manager;
use crate::config::Config;
use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Multipart},
    response::{Html, sse::{Event, KeepAlive, Sse}},
    http::StatusCode,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,  // "user" or "assistant"
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
    pub context_files: Option<Vec<ContextFileData>>,
    pub agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextFileData {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message_id: String,
    pub session_id: String,
    pub response: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub messages: Vec<ChatMessage>,
}

pub struct GuiState {
    pub manager: Arc<Manager>,
    pub sessions: Mutex<HashMap<String, Vec<ChatMessage>>>,
    pub event_channels: Mutex<HashMap<String, mpsc::UnboundedSender<String>>>,
    pub config: Mutex<Config>,
    pub config_path: Option<std::path::PathBuf>,
    pub task_handles: Mutex<HashMap<String, tokio::task::AbortHandle>>,
}

pub async fn start_gui_server(manager: Arc<Manager>, port: u16, config: Config) -> Result<()> {
    let config_path = Config::get_config_path().ok();
    let app = create_gui_app_with_path(manager, config, config_path);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("GUI server listening on http://0.0.0.0:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn create_gui_app(manager: Arc<Manager>, config: Config) -> Router {
    create_gui_app_with_path(manager, config, None)
}

pub fn create_gui_app_with_path(manager: Arc<Manager>, config: Config, config_path: Option<std::path::PathBuf>) -> Router {
    let state = Arc::new(GuiState {
        manager,
        sessions: Mutex::new(HashMap::new()),
        event_channels: Mutex::new(HashMap::new()),
        config: Mutex::new(config),
        config_path,
        task_handles: Mutex::new(HashMap::new()),
    });

    Router::new()
        .route("/", get(serve_index))
        .route("/api/chat", post(handle_chat))
        .route("/api/chat/stream", post(handle_chat_stream))
        .route("/api/session/:session_id", get(get_session))
        .route("/api/upload", post(handle_upload))
        .route("/api/events/:session_id", get(handle_events))
        .route("/api/settings", get(get_settings))
        .route("/api/settings", post(update_settings))
        .route("/api/task/stop/:session_id", post(stop_task))
        .route("/api/task/status/:session_id", get(get_task_status))
        .with_state(state)
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn handle_chat(
    State(state): State<Arc<GuiState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let session_id = request.session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let message_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    // Store user message
    let user_msg = ChatMessage {
        id: message_id.clone(),
        role: "user".to_string(),
        content: request.message.clone(),
        timestamp,
    };

    {
        let mut sessions = state.sessions.lock().await;
        sessions.entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push(user_msg);
    }

    // Convert context files to agent format
    let context_files = request.context_files
        .unwrap_or_default()
        .into_iter()
        .map(|cf| crate::agents::ContextFile {
            path: cf.path,
            content: cf.content,
        })
        .collect();

    // Run manager with selected agent
    let agent_name = request.agent.as_deref().unwrap_or("ProductOwner");
    let result = state.manager
        .run_with_session(
            agent_name,
            request.message,
            Some(session_id.clone()),
            context_files,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response_text = result.0;
    let response_timestamp = chrono::Utc::now().timestamp();

    // Store assistant response
    let assistant_msg = ChatMessage {
        id: Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: response_text.clone(),
        timestamp: response_timestamp,
    };

    {
        let mut sessions = state.sessions.lock().await;
        sessions.entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push(assistant_msg);
    }

    Ok(Json(ChatResponse {
        message_id: Uuid::new_v4().to_string(),
        session_id: session_id.clone(),
        response: response_text,
        timestamp: response_timestamp,
    }))
}

async fn handle_chat_stream(
    State(state): State<Arc<GuiState>>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = request.session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let message_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    // Store user message
    let user_msg = ChatMessage {
        id: message_id.clone(),
        role: "user".to_string(),
        content: request.message.clone(),
        timestamp,
    };

    {
        let mut sessions = state.sessions.lock().await;
        sessions.entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push(user_msg);
    }

    // Convert context files to agent format
    let context_files = request.context_files
        .unwrap_or_default()
        .into_iter()
        .map(|cf| crate::agents::ContextFile {
            path: cf.path,
            content: cf.content,
        })
        .collect();

    let (tx, rx) = mpsc::unbounded_channel();
    let manager = Arc::clone(&state.manager);
    let state_clone = Arc::clone(&state);
    let agent_name = request.agent.unwrap_or_else(|| "ProductOwner".to_string());
    let message = request.message.clone();
    let session_id_clone = session_id.clone();

    // Spawn background task to run agent and stream events
    let task_handle = tokio::spawn(async move {
        // Send session ID first
        let _ = tx.send(format!("session:{}", session_id));
        
        // Subscribe to agent events  
        let mut event_rx = manager.event_broadcaster.subscribe().await;
        
        // Clone tx for event forwarding
        let tx_events = tx.clone();
        
        // Spawn task to forward events - will stop when channel closes or no events for 100ms
        let event_handle = tokio::spawn(async move {
            loop {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(100),
                    event_rx.recv()
                ).await {
                    Ok(Some(event)) => {
                        let event_json = match serde_json::to_string(&event) {
                            Ok(json) => json,
                            Err(e) => {
                                tracing::warn!("Failed to serialize event: {}", e);
                                continue;
                            }
                        };
                        // If send fails, receiver is dropped, exit
                        if tx_events.send(format!("event:{}", event_json)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => {
                        // Channel closed, exit
                        break;
                    }
                    Err(_) => {
                        // Timeout - check if we should continue or exit
                        // For now, just continue waiting for more events
                        continue;
                    }
                }
            }
        });
        
        // Run manager
        match manager.run_with_session(
            &agent_name,
            message,
            Some(session_id.clone()),
            context_files,
        ).await {
            Ok(result) => {
                let response_text = result.0;
                let response_timestamp = chrono::Utc::now().timestamp();

                // Store assistant response
                let assistant_msg = ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    role: "assistant".to_string(),
                    content: response_text.clone(),
                    timestamp: response_timestamp,
                };

                {
                    let mut sessions = state_clone.sessions.lock().await;
                    sessions.entry(session_id.clone())
                        .or_insert_with(Vec::new)
                        .push(assistant_msg);
                }

                // Send complete response
                let _ = tx.send(format!("data:{}", response_text));
                let _ = tx.send("done".to_string());
            }
            Err(e) => {
                // Send error and then done signal so UI knows stream is complete
                let _ = tx.send(format!("error:{}", e));
                let _ = tx.send("done".to_string());
            }
        }
        
        // Clean up task handle
        {
            let mut handles = state_clone.task_handles.lock().await;
            handles.remove(&session_id);
        }
        
        // Give event task a moment to process any remaining events, then abort
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        event_handle.abort();
    });
    
    // Store the task handle
    {
        let mut handles = state.task_handles.lock().await;
        handles.insert(session_id_clone, task_handle.abort_handle());
    }

    // Convert receiver to stream
    let stream = UnboundedReceiverStream::new(rx);
    let event_stream = stream.map(|msg| Ok(Event::default().data(msg)));

    Sse::new(event_stream).keep_alive(KeepAlive::default())
}

async fn get_session(
    State(state): State<Arc<GuiState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<SessionInfo>, (StatusCode, String)> {
    let sessions = state.sessions.lock().await;
    let messages = sessions.get(&session_id)
        .cloned()
        .unwrap_or_default();

    Ok(Json(SessionInfo {
        session_id,
        messages,
    }))
}

async fn handle_upload(
    State(_state): State<Arc<GuiState>>,
    multipart: Multipart,
) -> Result<Json<Vec<ContextFileData>>, (StatusCode, String)> {
    let mut files = Vec::new();
    let mut multipart = multipart;

    loop {
        let field_result = multipart.next_field().await;
        
        match field_result {
            Ok(Some(field)) => {
                let name: String = match field.file_name() {
                    Some(s) => s.to_string(),
                    None => "unnamed".to_string(),
                };
                
                let bytes_result = field.bytes().await;
                let data: Bytes = match bytes_result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return Err((StatusCode::BAD_REQUEST, format!("Failed to read bytes: {}", e)))
                    },
                };
                
                let content = String::from_utf8(data.to_vec())
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", e)))?;

                files.push(ContextFileData {
                    path: name,
                    content,
                });
            },
            Ok(None) => break,
            Err(e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Failed to get field: {}", e)))
            },
        }
    }

    Ok(Json(files))
}

async fn handle_events(
    State(_state): State<Arc<GuiState>>,
    axum::extract::Path(_session_id): axum::extract::Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Send initial connected event
    let _ = tx.send("connected".to_string());
    
    // Convert receiver to stream
    let stream = UnboundedReceiverStream::new(rx);
    let event_stream = stream.map(|msg| Ok(Event::default().data(msg)));

    Sse::new(event_stream).keep_alive(KeepAlive::default())
}

async fn get_settings(
    State(state): State<Arc<GuiState>>,
) -> Json<Config> {
    let config = state.config.lock().await;
    Json(config.clone())
}

async fn update_settings(
    State(state): State<Arc<GuiState>>,
    Json(new_config): Json<Config>,
) -> Result<Json<Config>, (StatusCode, String)> {
    let mut config = state.config.lock().await;
    *config = new_config.clone();
    
    // Save to disk if path is provided
    if let Some(path) = &state.config_path {
        if let Err(e) = config.save_to_path(path.clone()) {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save config to {:?}: {}", path, e)));
        }
    }
    
    Ok(Json(new_config))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub is_running: bool,
    pub session_id: String,
}

async fn stop_task(
    State(state): State<Arc<GuiState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<TaskStatusResponse>, (StatusCode, String)> {
    let mut handles = state.task_handles.lock().await;
    
    if let Some(handle) = handles.remove(&session_id) {
        handle.abort();
        tracing::info!("Task stopped for session: {}", session_id);
        Ok(Json(TaskStatusResponse {
            is_running: false,
            session_id,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, format!("No running task found for session: {}", session_id)))
    }
}

async fn get_task_status(
    State(state): State<Arc<GuiState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Json<TaskStatusResponse> {
    let handles = state.task_handles.lock().await;
    let is_running = handles.contains_key(&session_id);
    
    Json(TaskStatusResponse {
        is_running,
        session_id,
    })
}
