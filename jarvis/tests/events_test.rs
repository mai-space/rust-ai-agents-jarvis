/// Tests for the event system (events.rs)
/// 
/// The event system provides real-time feedback from agents to GUI and other consumers.
/// These tests verify:
/// - Event broadcasting works correctly
/// - Multiple subscribers receive events
/// - TaskSummary tracks file operations and agents correctly
/// - Markdown generation from TaskSummary is accurate

use jarvis::events::{EventBroadcaster, TaskSummary, AgentEvent, FileOpType};
use tokio::time::{timeout, Duration};

/// Test that TaskSummary correctly tracks file operations
/// 
/// This ensures that:
/// - Created files are tracked without duplicates
/// - Modified files are tracked without duplicates
/// - Deleted files are tracked without duplicates
/// - Read operations are ignored (not tracked in summary)
#[test]
fn test_task_summary_file_operations() {
    let mut summary = TaskSummary::new();
    
    // Add various file operations
    summary.add_file_operation(&FileOpType::Created, "src/main.rs".to_string());
    summary.add_file_operation(&FileOpType::Modified, "src/lib.rs".to_string());
    summary.add_file_operation(&FileOpType::Deleted, "old_file.rs".to_string());
    summary.add_file_operation(&FileOpType::Read, "config.toml".to_string());
    
    // Test no duplicates
    summary.add_file_operation(&FileOpType::Created, "src/main.rs".to_string());
    summary.add_file_operation(&FileOpType::Modified, "src/lib.rs".to_string());
    
    assert_eq!(summary.files_created.len(), 1);
    assert_eq!(summary.files_modified.len(), 1);
    assert_eq!(summary.files_deleted.len(), 1);
    assert!(summary.files_created.contains(&"src/main.rs".to_string()));
    assert!(summary.files_modified.contains(&"src/lib.rs".to_string()));
    assert!(summary.files_deleted.contains(&"old_file.rs".to_string()));
}

/// Test that TaskSummary tracks agents without duplicates
/// 
/// Verifies that the same agent name is only added once to the list
/// of agents involved in the task.
#[test]
fn test_task_summary_agents() {
    let mut summary = TaskSummary::new();
    
    summary.add_agent("ProductOwner".to_string());
    summary.add_agent("RequirementsEngineer".to_string());
    summary.add_agent("ProductOwner".to_string()); // Duplicate
    
    assert_eq!(summary.agents_involved.len(), 2);
    assert!(summary.agents_involved.contains(&"ProductOwner".to_string()));
    assert!(summary.agents_involved.contains(&"RequirementsEngineer".to_string()));
}

/// Test TaskSummary markdown generation
/// 
/// Ensures that the markdown output includes:
/// - Proper headers
/// - Agent list
/// - File changes categorized by operation type
/// - Duration if available
#[test]
fn test_task_summary_markdown() {
    let mut summary = TaskSummary::new();
    summary.description = "Test task completed successfully".to_string();
    summary.add_agent("ProductOwner".to_string());
    summary.add_agent("SeniorDeveloper".to_string());
    summary.add_file_operation(&FileOpType::Created, "new_file.rs".to_string());
    summary.add_file_operation(&FileOpType::Modified, "existing_file.rs".to_string());
    summary.total_duration_ms = Some(1500);
    
    let markdown = summary.to_markdown();
    
    assert!(markdown.contains("# Task Summary"));
    assert!(markdown.contains("Test task completed successfully"));
    assert!(markdown.contains("ProductOwner"));
    assert!(markdown.contains("SeniorDeveloper"));
    assert!(markdown.contains("### Created"));
    assert!(markdown.contains("`new_file.rs`"));
    assert!(markdown.contains("### Modified"));
    assert!(markdown.contains("`existing_file.rs`"));
    assert!(markdown.contains("**Duration:** 1500ms"));
}

/// Test markdown generation for empty TaskSummary
/// 
/// Verifies that an empty task summary generates valid markdown
/// with appropriate "no changes" messages.
#[test]
fn test_task_summary_markdown_empty() {
    let summary = TaskSummary::new();
    let markdown = summary.to_markdown();
    
    assert!(markdown.contains("# Task Summary"));
    assert!(markdown.contains("## Agents Involved"));
    assert!(markdown.contains("- None"));
    assert!(markdown.contains("No files were changed"));
}

/// Test event broadcaster with single subscriber
/// 
/// Verifies that:
/// - A subscriber can receive events
/// - Events are delivered correctly
/// - Event data is preserved during transmission
#[tokio::test]
async fn test_event_broadcaster_single_subscriber() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver = broadcaster.subscribe().await;
    
    // Send an agent started event
    broadcaster.agent_started("ProductOwner".to_string()).await;
    
    // Receive and verify the event
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed unexpectedly");
    
    match event {
        AgentEvent::AgentStarted { agent_name, .. } => {
            assert_eq!(agent_name, "ProductOwner");
        }
        _ => panic!("Expected AgentStarted event"),
    }
}

/// Test event broadcaster with multiple subscribers
/// 
/// Ensures that:
/// - Multiple subscribers can listen simultaneously
/// - All subscribers receive the same events
/// - Events are broadcast to all active subscribers
#[tokio::test]
async fn test_event_broadcaster_multiple_subscribers() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver1 = broadcaster.subscribe().await;
    let mut receiver2 = broadcaster.subscribe().await;
    let mut receiver3 = broadcaster.subscribe().await;
    
    // Broadcast a tool call event
    broadcaster.tool_call(
        "SeniorDeveloper".to_string(),
        "write_file".to_string(),
        "Writing code to file".to_string(),
    ).await;
    
    // All subscribers should receive the event
    for receiver in [&mut receiver1, &mut receiver2, &mut receiver3] {
        let event = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed unexpectedly");
        
        match event {
            AgentEvent::ToolCall { agent_name, tool_name, .. } => {
                assert_eq!(agent_name, "SeniorDeveloper");
                assert_eq!(tool_name, "write_file");
            }
            _ => panic!("Expected ToolCall event"),
        }
    }
}

/// Test event broadcaster removes closed subscribers
/// 
/// Verifies that when a subscriber closes their receiver,
/// the broadcaster removes it from the active subscriber list
/// and continues to work with remaining subscribers.
#[tokio::test]
async fn test_event_broadcaster_closed_subscriber() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver1 = broadcaster.subscribe().await;
    let receiver2 = broadcaster.subscribe().await;
    
    // Close receiver2 immediately
    drop(receiver2);
    
    // Broadcast event
    broadcaster.agent_thought(
        "QATester".to_string(),
        "Analyzing test coverage".to_string(),
    ).await;
    
    // receiver1 should still receive the event
    let event = timeout(Duration::from_secs(1), receiver1.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed unexpectedly");
    
    match event {
        AgentEvent::AgentThought { agent_name, thought, .. } => {
            assert_eq!(agent_name, "QATester");
            assert_eq!(thought, "Analyzing test coverage");
        }
        _ => panic!("Expected AgentThought event"),
    }
}

/// Test all event types are broadcast correctly
/// 
/// Comprehensive test that verifies all event types defined in the system
/// can be created and broadcast successfully.
#[tokio::test]
async fn test_all_event_types() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver = broadcaster.subscribe().await;
    
    // Test handoff event
    broadcaster.handoff(
        "ProductOwner".to_string(),
        "RequirementsEngineer".to_string(),
        "Project analysis complete".to_string(),
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(event, AgentEvent::Handoff { .. }));
    
    // Test task completed event
    broadcaster.task_completed(
        "Librarian".to_string(),
        "Documentation finalized".to_string(),
        Some(TaskSummary::new()),
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(event, AgentEvent::TaskCompleted { .. }));
    
    // Test task failed event
    broadcaster.task_failed(
        "SeniorDeveloper".to_string(),
        "Compilation error".to_string(),
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(event, AgentEvent::TaskFailed { .. }));
    
    // Test file operation event
    broadcaster.file_operation(FileOpType::Created, "test.rs".to_string()).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(event, AgentEvent::FileOperation { .. }));
    
    // Test loop detected event
    broadcaster.loop_detected(vec![
        "Agent1".to_string(),
        "Agent2".to_string(),
    ]).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(event, AgentEvent::LoopDetected { .. }));
    
    // Test human intervention requested event
    broadcaster.human_intervention_requested(
        "SecurityExpert".to_string(),
        "Critical security issue found".to_string(),
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(event, AgentEvent::HumanInterventionRequested { .. }));
}

/// Test tool result event with success/failure
/// 
/// Verifies that tool execution results are properly tracked
/// with their success/failure status.
#[tokio::test]
async fn test_tool_result_events() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver = broadcaster.subscribe().await;
    
    // Test successful tool result
    broadcaster.tool_result(
        "SeniorDeveloper".to_string(),
        "run_tests".to_string(),
        "All tests passed".to_string(),
        true,
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    
    match event {
        AgentEvent::ToolResult { success, tool_name, .. } => {
            assert!(success);
            assert_eq!(tool_name, "run_tests");
        }
        _ => panic!("Expected ToolResult event"),
    }
    
    // Test failed tool result
    broadcaster.tool_result(
        "QATester".to_string(),
        "static_analysis".to_string(),
        "Linting errors found".to_string(),
        false,
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    
    match event {
        AgentEvent::ToolResult { success, tool_name, .. } => {
            assert!(!success);
            assert_eq!(tool_name, "static_analysis");
        }
        _ => panic!("Expected ToolResult event"),
    }
}

/// Test plan created event
/// 
/// Verifies that plan creation events are properly broadcast
/// with the complete plan data.
#[tokio::test]
async fn test_plan_created_event() {
    let broadcaster = EventBroadcaster::new();
    let mut receiver = broadcaster.subscribe().await;
    
    let plan = "1. Analyze requirements\n2. Design architecture\n3. Implement features";
    
    broadcaster.plan_created(
        "RequirementsEngineer".to_string(),
        plan.to_string(),
    ).await;
    
    let event = timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    
    match event {
        AgentEvent::PlanCreated { agent_name, plan: received_plan, .. } => {
            assert_eq!(agent_name, "RequirementsEngineer");
            assert_eq!(received_plan, plan);
        }
        _ => panic!("Expected PlanCreated event"),
    }
}
