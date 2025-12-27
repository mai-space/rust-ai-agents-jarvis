use crate::orchestration::Manager;
use axum::{
    routing::post,
    Router, Json, extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub task_id: String,
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskInput {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Step {
    pub step_id: String,
    pub task_id: String,
    pub status: String,
    pub output: Option<String>,
}

pub struct AcpState {
    pub manager: Arc<Manager>,
    pub tasks: Mutex<HashMap<String, Task>>,
}

pub async fn start_acp_server(manager: Arc<Manager>, port: u16) -> Result<()> {
    let app = create_app(manager);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn create_app(manager: Arc<Manager>) -> Router {
    let state = Arc::new(AcpState {
        manager,
        tasks: Mutex::new(HashMap::new()),
    });

    Router::new()
        .route("/agent/tasks", post(create_task))
        .route("/agent/tasks/:task_id/steps", post(execute_step))
        .with_state(state)
}

async fn create_task(
    State(state): State<Arc<AcpState>>,
    Json(input): Json<TaskInput>,
) -> Json<Task> {
    let task_id = Uuid::new_v4().to_string();
    let task = Task {
        task_id: task_id.clone(),
        input: input.input,
    };
    
    state.tasks.lock().await.insert(task_id.clone(), task.clone());
    Json(task)
}

async fn execute_step(
    Path(task_id): Path<String>,
    State(state): State<Arc<AcpState>>,
) -> Json<Step> {
    let tasks = state.tasks.lock().await;
    let task = match tasks.get(&task_id) {
        Some(t) => t,
        None => return Json(Step {
            step_id: Uuid::new_v4().to_string(),
            task_id,
            status: "error".to_string(),
            output: Some("Task not found".to_string()),
        }),
    };

    // In a real Agent Protocol, steps might be granular.
    // For Jarvis, we'll run the whole manager cycle for now as one "step" 
    // or we could refactor manager to be step-based.
    // To satisfy the protocol quickly, we'll just run it.
    
    match state.manager.run("ProductOwner", task.input.clone()).await {
        Ok(result) => Json(Step {
            step_id: Uuid::new_v4().to_string(),
            task_id: task_id.clone(),
            status: "completed".to_string(),
            output: Some(result),
        }),
        Err(e) => Json(Step {
            step_id: Uuid::new_v4().to_string(),
            task_id: task_id.clone(),
            status: "failed".to_string(),
            output: Some(e.to_string()),
        }),
    }
}
