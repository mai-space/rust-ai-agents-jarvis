# Agent Architecture Overhaul

This document describes the major architectural improvements made to the Jarvis AI Agent Squad to address issues with agent confusion, poor feedback, and lack of visibility into agent operations.

## Problem Statement

The original issue showed agents getting stuck in loops and exceeding maximum interaction steps:

```
2025-12-28T12:53:32.507739Z  INFO jarvis::agents: Agent Thought: THOUGHT:  I need to create a new Github Actions workflow file named "installers.yml" within the ".github/workflows" directory.
2025-12-28T12:53:32.507758Z ERROR jarvis::orchestration: Manager: Error from agent 'RequirementsEngineer': Agent exceeded maximum interaction steps
```

### Key Issues Identified

1. **Agents exceeded max steps** - Both ProductOwner and RequirementsEngineer were hitting the 20-step limit
2. **No UI feedback** - Users couldn't see what agents were thinking or doing
3. **RequirementsEngineer had no tools** - It couldn't examine files, leading to confusion
4. **No task summaries** - At task end, there was no clear summary of what was accomplished
5. **Poor handoff guidance** - Agents weren't clear on their roles and when to hand off

## Solution: Multi-Faceted Architecture Improvements

### 1. Real-Time Event System

#### Event Types
Created a comprehensive event system (`jarvis/src/events.rs`) with the following event types:

- **AgentStarted** - When an agent begins processing
- **AgentThought** - Agent's reasoning and thinking
- **ToolCall** - When an agent calls a tool
- **ToolResult** - Result of tool execution
- **PlanCreated** - When an agent creates a structured plan
- **Handoff** - Agent transition information
- **TaskCompleted** - Task finished successfully with summary
- **TaskFailed** - Task failed with error details
- **FileOperation** - File create/modify/delete tracking
- **LoopDetected** - When agent loops are detected
- **HumanInterventionRequested** - When human input is needed

#### EventBroadcaster

The `EventBroadcaster` allows multiple subscribers to receive events:

```rust
pub struct EventBroadcaster {
    senders: Arc<tokio::sync::RwLock<Vec<mpsc::UnboundedSender<AgentEvent>>>>,
}
```

Key methods:
- `subscribe()` - Get a receiver for events
- `broadcast()` - Send event to all subscribers
- Helper methods for each event type (e.g., `agent_started()`, `tool_call()`, etc.)

#### Integration

1. **Manager** - Manager has an `EventBroadcaster` and emits events for:
   - Loop detection
   - Human intervention requests
   - Task completion/failure
   - Handoffs

2. **AgentContext** - Each agent context has access to the event broadcaster to emit:
   - Agent thoughts
   - Tool calls and results
   - Plan creation

3. **GUI** - The GUI SSE endpoint subscribes to events and streams them to the frontend

### 2. Task Summary System

#### TaskSummary Structure

```rust
pub struct TaskSummary {
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
    pub files_deleted: Vec<String>,
    pub agents_involved: Vec<String>,
    pub total_duration_ms: Option<u64>,
    pub description: String,
}
```

Features:
- Tracks all file operations
- Records agents involved
- Measures total duration
- Can generate markdown summary with `to_markdown()`

#### Integration

- `AgentContext` includes a shared `Arc<RwLock<TaskSummary>>`
- Agents are automatically tracked when they start processing
- File operations can be recorded (infrastructure in place)
- Final summary included in `TaskCompleted` event

### 3. Improved Agent Instructions

#### ProductOwner
**Changes:**
- Added explicit workflow steps
- Introduced PLAN command for creating visible plans
- Set expectation of 2-4 tool calls max before handoff
- Clear guidance on when to hand off vs continue

**Key improvement:**
```rust
"2. CREATE PLAN: Use the PLAN command to create a structured markdown plan with clear sections:
   - Overview: What needs to be done
   - Key Files: List relevant files
   - Approach: High-level strategy
   - Success Criteria: How we know it's done"
```

#### RequirementsEngineer
**Changes:**
- **Now has read-only tools**: `read_file`, `list_files`, `read_structure`
- Can examine files mentioned by ProductOwner
- Given explicit workflow: Review → Create Detailed Plan → Handoff
- Set expectation of 3-5 tool calls max

**Why this matters:** Previously had NO tools, leading to confusion and repeated failed attempts to access information.

#### Librarian
**Changes:**
- Clarified dual role:
  1. **Context Provider** - Provides project history/preferences when asked
  2. **Task Finalizer** - Creates final summary and documentation
- Added responsibility for summary generation
- Should check task summary for file changes
- Use `store_preference` for patterns to remember

### 4. PLAN Command

Added new `PLAN` command that agents can use to create structured plans:

```
PLAN <markdown_plan>
```

Benefits:
- Makes plans visible to UI
- Shared with other agents via events
- Helps agents think through approach before acting
- Creates documentation of intent

### 5. Increased Interaction Limits

Changed max interaction steps:
- **With tools**: 30 steps (was 20)
- **Without tools**: 15 steps (was 20 for all)

Reasoning:
- Agents with tools need more iterations to explore
- Agents without tools should hand off faster
- Still prevents infinite loops

### 6. GUI Event Streaming

Updated `handle_chat_stream` in `gui.rs` to:

1. Subscribe to event broadcaster
2. Spawn task to forward events via SSE
3. Stream events with format: `event:{json}`

Frontend receives:
```
session:{session_id}
event:{"type":"agent_started","agent_name":"ProductOwner",...}
event:{"type":"agent_thought","thought":"I should check the structure",...}
event:{"type":"tool_call","tool_name":"read_structure",...}
...
data:{final_response}
done
```

## Implementation Details

### Changes to Core Files

#### `jarvis/src/events.rs` (NEW)
- Complete event system
- 11 event types
- EventBroadcaster for pub/sub
- TaskSummary with markdown formatting

#### `jarvis/src/agents/mod.rs`
- Added event emission in `run_llm_agent`
- Added PLAN command handler
- Increased max steps with tool-based logic
- Updated `AgentContext` with event_broadcaster and task_summary

#### `jarvis/src/orchestration/mod.rs`
- Added `EventBroadcaster` to `Manager`
- Initialize context with broadcaster and summary
- Emit events for loops, failures, completions, handoffs
- Track task duration

#### `jarvis/src/orchestration/gui.rs`
- Subscribe to events in `handle_chat_stream`
- Forward events via SSE to frontend
- Events format: `event:{json}`

#### `jarvis/src/agents/planning.rs`
- Updated ProductOwner identity with workflow
- Added tools to RequirementsEngineer
- Clearer handoff instructions

#### `jarvis/src/agents/documentation.rs`
- Enhanced Librarian with dual role description
- Added summary generation responsibility
- Check task summary for file changes

#### `jarvis/src/main.rs`
- Instantiate RequirementsEngineer with read-only tools

## Testing

### Test Updates
Updated all tests to include new AgentContext fields:
- `event_broadcaster: None` (tests don't need events)
- `task_summary: Arc::new(RwLock::new(TaskSummary::new()))`

Updated files:
- `jarvis/tests/integration_test.rs`
- `jarvis/tests/phase3_test.rs`
- `jarvis/tests/phase7_test.rs`
- `jarvis/tests/phase8_test.rs`

### Verification
- ✅ All library tests pass (32 tests)
- ✅ Build succeeds with no errors
- ✅ Integration tests compile successfully

## Usage

### For Agent Developers

When creating new agents, use the event broadcaster:

```rust
// In your agent's process method
if let Some(broadcaster) = &context.event_broadcaster {
    broadcaster.agent_thought(
        agent_name.clone(), 
        "I'm analyzing the file structure".to_string()
    ).await;
}
```

### For GUI Developers

Frontend can listen to SSE events and parse them:

```javascript
const eventSource = new EventSource('/api/chat/stream');

eventSource.addEventListener('message', (e) => {
    if (e.data.startsWith('event:')) {
        const event = JSON.parse(e.data.substring(6));
        // Handle different event types
        switch(event.type) {
            case 'agent_thought':
                displayThought(event.agent_name, event.thought);
                break;
            case 'tool_call':
                displayToolCall(event.tool_name);
                break;
            // ... etc
        }
    }
});
```

## Benefits

### For Users
1. **Real-time visibility** - See what agents are thinking and doing
2. **Better feedback** - Know when agents are stuck or progressing
3. **Clear summaries** - Understand what changed at task completion
4. **Transparency** - See the full agent workflow

### For Agents
1. **More tools** - RequirementsEngineer can now examine files
2. **Clearer roles** - Each agent knows exactly what to do
3. **Better limits** - More reasonable iteration counts
4. **Plan sharing** - PLAN command makes intent visible

### For Developers
1. **Debugging** - Events provide insight into agent behavior
2. **Monitoring** - Track agent performance and issues
3. **Extensibility** - Easy to add new event types
4. **Testing** - Can verify agent actions through events

## Future Enhancements

### Short Term
1. **Frontend UI** - Update `static/index.html` to display events
2. **File tracking** - Automatically detect file operations from tool calls
3. **Plan visualization** - Render markdown plans in UI
4. **Summary view** - Dedicated UI for task completion summary

### Medium Term
1. **Event persistence** - Store events in database for history
2. **Agent metrics** - Track performance by agent/tool
3. **Event filtering** - Allow users to filter event types
4. **Replay** - Ability to replay task execution from events

### Long Term
1. **Agent learning** - Use event history to improve agent behavior
2. **Custom agents** - User-defined agents with event support
3. **Distributed agents** - Events across multiple machines
4. **Real-time collaboration** - Multiple users watching same task

## Migration Guide

### For Existing Code

If you have custom agents or tools:

1. **Update AgentContext creation:**
   ```rust
   // Old
   let context = AgentContext {
       task, history, vector_db, available_agents, 
       project_metadata, handoff_count, context_files,
   };
   
   // New
   let context = AgentContext {
       task, history, vector_db, available_agents,
       project_metadata, handoff_count, context_files,
       event_broadcaster: Some(broadcaster),
       task_summary: Arc::new(RwLock::new(TaskSummary::new())),
   };
   ```

2. **Update RequirementsEngineer instantiation:**
   ```rust
   // Old
   let re = Arc::new(RequirementsEngineer::new(llm));
   
   // New
   let re_tools = vec![
       Arc::new(ReadFileTool),
       Arc::new(ListFilesTool),
       Arc::new(ReadStructureTool),
   ];
   let re = Arc::new(RequirementsEngineer::new(llm, re_tools));
   ```

3. **Optional: Emit events in your custom agents**
   ```rust
   if let Some(broadcaster) = &context.event_broadcaster {
       broadcaster.agent_thought(name, thought).await;
   }
   ```

## Conclusion

This overhaul significantly improves the Jarvis agent system by:
- Adding comprehensive event streaming for visibility
- Giving agents appropriate tools for their roles
- Providing clearer instructions and workflows
- Implementing task summaries for completion reporting
- Enabling real-time UI feedback

The architecture is now more transparent, easier to debug, and provides better user experience while maintaining the autonomous multi-agent approach that makes Jarvis powerful.

## References

- Event System: `jarvis/src/events.rs`
- Agent Updates: `jarvis/src/agents/mod.rs`, `jarvis/src/agents/planning.rs`, `jarvis/src/agents/documentation.rs`
- Orchestration: `jarvis/src/orchestration/mod.rs`, `jarvis/src/orchestration/gui.rs`
- Main Integration: `jarvis/src/main.rs`
