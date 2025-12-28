# Frontend Integration Guide for Agent Events

This guide explains how to update the Jarvis GUI frontend to display real-time agent events.

## Event Stream Format

The `/api/chat/stream` endpoint now emits three types of messages:

1. **Session ID**: `session:{uuid}`
2. **Agent Events**: `event:{json}`
3. **Final Response**: `data:{response_text}`
4. **Completion**: `done`

## Event Types

All events are JSON objects with a `type` field:

### 1. agent_started
```json
{
  "type": "agent_started",
  "agent_name": "ProductOwner",
  "timestamp": 1703779200
}
```

### 2. agent_thought
```json
{
  "type": "agent_thought",
  "agent_name": "ProductOwner",
  "thought": "I need to understand the project structure first",
  "timestamp": 1703779201
}
```

### 3. tool_call
```json
{
  "type": "tool_call",
  "agent_name": "ProductOwner",
  "tool_name": "read_structure",
  "input_summary": "{ path: \".\" }",
  "timestamp": 1703779202
}
```

### 4. tool_result
```json
{
  "type": "tool_result",
  "agent_name": "ProductOwner",
  "tool_name": "read_structure",
  "output_summary": "structure with 12 top-level items",
  "success": true,
  "timestamp": 1703779203
}
```

### 5. plan_created
```json
{
  "type": "plan_created",
  "agent_name": "ProductOwner",
  "plan": "# Implementation Plan\n\n## Overview\n...",
  "timestamp": 1703779204
}
```

### 6. handoff
```json
{
  "type": "handoff",
  "from_agent": "ProductOwner",
  "to_agent": "RequirementsEngineer",
  "reason": "InitialPlanReady",
  "timestamp": 1703779205
}
```

### 7. task_completed
```json
{
  "type": "task_completed",
  "agent_name": "Librarian",
  "result": "Task completed successfully",
  "summary": {
    "files_created": ["src/new_file.rs"],
    "files_modified": ["src/main.rs"],
    "files_deleted": [],
    "agents_involved": ["ProductOwner", "RequirementsEngineer", "SeniorDeveloper", "Librarian"],
    "total_duration_ms": 45000,
    "description": "Created new feature with proper documentation"
  },
  "timestamp": 1703779300
}
```

### 8. task_failed
```json
{
  "type": "task_failed",
  "agent_name": "SeniorDeveloper",
  "error": "File not found: missing.rs",
  "timestamp": 1703779250
}
```

### 9. loop_detected
```json
{
  "type": "loop_detected",
  "agents": ["ProductOwner", "RequirementsEngineer"],
  "timestamp": 1703779220
}
```

### 10. human_intervention_requested
```json
{
  "type": "human_intervention_requested",
  "agent_name": "RequirementsEngineer",
  "reason": "Loop detected between agents",
  "timestamp": 1703779221
}
```

## Example JavaScript Implementation

### Basic Event Handling

```javascript
function connectToAgentStream(message, sessionId = null) {
  const eventSource = new EventSource('/api/chat/stream', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message, session_id: sessionId })
  });

  let currentSessionId = null;

  eventSource.addEventListener('message', (e) => {
    const data = e.data;

    if (data.startsWith('session:')) {
      currentSessionId = data.substring(8);
      console.log('Session ID:', currentSessionId);
      
    } else if (data.startsWith('event:')) {
      const event = JSON.parse(data.substring(6));
      handleAgentEvent(event);
      
    } else if (data.startsWith('data:')) {
      const response = data.substring(5);
      displayFinalResponse(response);
      
    } else if (data === 'done') {
      eventSource.close();
      showCompleted();
    }
  });

  eventSource.addEventListener('error', (e) => {
    console.error('Stream error:', e);
    eventSource.close();
  });
}
```

### Event Handler

```javascript
function handleAgentEvent(event) {
  const container = document.getElementById('agent-events');
  
  switch(event.type) {
    case 'agent_started':
      appendEvent(container, {
        icon: '🤖',
        title: `${event.agent_name} Started`,
        time: formatTime(event.timestamp),
        class: 'agent-start'
      });
      break;

    case 'agent_thought':
      appendEvent(container, {
        icon: '💭',
        title: `${event.agent_name} is thinking`,
        content: event.thought,
        time: formatTime(event.timestamp),
        class: 'agent-thought'
      });
      break;

    case 'tool_call':
      appendEvent(container, {
        icon: '🔧',
        title: `${event.agent_name} calling ${event.tool_name}`,
        content: event.input_summary,
        time: formatTime(event.timestamp),
        class: 'tool-call'
      });
      break;

    case 'tool_result':
      const icon = event.success ? '✅' : '❌';
      appendEvent(container, {
        icon: icon,
        title: `${event.tool_name} result`,
        content: event.output_summary,
        time: formatTime(event.timestamp),
        class: event.success ? 'tool-success' : 'tool-error'
      });
      break;

    case 'plan_created':
      appendEvent(container, {
        icon: '📋',
        title: `${event.agent_name} created a plan`,
        content: renderMarkdown(event.plan),
        time: formatTime(event.timestamp),
        class: 'plan-created',
        expandable: true
      });
      break;

    case 'handoff':
      appendEvent(container, {
        icon: '🔄',
        title: `Handoff: ${event.from_agent} → ${event.to_agent}`,
        content: event.reason,
        time: formatTime(event.timestamp),
        class: 'handoff'
      });
      break;

    case 'task_completed':
      displayTaskSummary(event.summary);
      break;

    case 'task_failed':
      appendEvent(container, {
        icon: '❌',
        title: `Task Failed`,
        content: `${event.agent_name}: ${event.error}`,
        time: formatTime(event.timestamp),
        class: 'task-failed'
      });
      break;

    case 'loop_detected':
      appendEvent(container, {
        icon: '⚠️',
        title: 'Loop Detected',
        content: `Agents stuck in loop: ${event.agents.join(' ↔ ')}`,
        time: formatTime(event.timestamp),
        class: 'loop-warning'
      });
      break;

    case 'human_intervention_requested':
      showHumanInterventionDialog(event);
      break;
  }
}
```

### UI Components

```javascript
function appendEvent(container, options) {
  const eventDiv = document.createElement('div');
  eventDiv.className = `agent-event ${options.class}`;
  
  let html = `
    <div class="event-header">
      <span class="event-icon">${options.icon}</span>
      <span class="event-title">${options.title}</span>
      <span class="event-time">${options.time}</span>
    </div>
  `;
  
  if (options.content) {
    if (options.expandable) {
      html += `
        <div class="event-content collapsed" onclick="this.classList.toggle('collapsed')">
          ${options.content}
        </div>
      `;
    } else {
      html += `<div class="event-content">${options.content}</div>`;
    }
  }
  
  eventDiv.innerHTML = html;
  container.appendChild(eventDiv);
  
  // Auto-scroll to bottom
  container.scrollTop = container.scrollHeight;
}

function displayTaskSummary(summary) {
  const summaryDiv = document.getElementById('task-summary');
  
  let html = '<div class="summary-section">';
  html += '<h3>📊 Task Summary</h3>';
  
  if (summary.description) {
    html += `<p>${summary.description}</p>`;
  }
  
  if (summary.agents_involved.length > 0) {
    html += '<h4>Agents Involved</h4>';
    html += '<ul>' + summary.agents_involved.map(a => `<li>${a}</li>`).join('') + '</ul>';
  }
  
  if (summary.files_created.length > 0) {
    html += '<h4>✨ Files Created</h4>';
    html += '<ul>' + summary.files_created.map(f => `<li><code>${f}</code></li>`).join('') + '</ul>';
  }
  
  if (summary.files_modified.length > 0) {
    html += '<h4>✏️ Files Modified</h4>';
    html += '<ul>' + summary.files_modified.map(f => `<li><code>${f}</code></li>`).join('') + '</ul>';
  }
  
  if (summary.files_deleted.length > 0) {
    html += '<h4>🗑️ Files Deleted</h4>';
    html += '<ul>' + summary.files_deleted.map(f => `<li><code>${f}</code></li>`).join('') + '</ul>';
  }
  
  if (summary.total_duration_ms) {
    const duration = (summary.total_duration_ms / 1000).toFixed(2);
    html += `<p><strong>Duration:</strong> ${duration}s</p>`;
  }
  
  html += '</div>';
  summaryDiv.innerHTML = html;
}

function formatTime(timestamp) {
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString();
}

function renderMarkdown(text) {
  // Simple markdown rendering - replace with proper library in production
  return text
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/\n/g, '<br>');
}
```

### CSS Styling

```css
.agent-events {
  max-height: 500px;
  overflow-y: auto;
  padding: 1rem;
  background: #f5f5f5;
  border-radius: 8px;
}

.agent-event {
  margin-bottom: 1rem;
  padding: 0.75rem;
  background: white;
  border-radius: 4px;
  border-left: 3px solid #ccc;
}

.agent-event.agent-start {
  border-left-color: #4CAF50;
}

.agent-event.agent-thought {
  border-left-color: #2196F3;
}

.agent-event.tool-call {
  border-left-color: #FF9800;
}

.agent-event.tool-success {
  border-left-color: #4CAF50;
}

.agent-event.tool-error {
  border-left-color: #F44336;
}

.agent-event.plan-created {
  border-left-color: #9C27B0;
}

.agent-event.handoff {
  border-left-color: #00BCD4;
  font-weight: 500;
}

.agent-event.task-failed {
  border-left-color: #F44336;
  background: #FFEBEE;
}

.agent-event.loop-warning {
  border-left-color: #FFC107;
  background: #FFF9C4;
}

.event-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.event-icon {
  font-size: 1.25rem;
}

.event-title {
  flex: 1;
  font-weight: 500;
}

.event-time {
  font-size: 0.875rem;
  color: #666;
}

.event-content {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: #f9f9f9;
  border-radius: 4px;
  font-size: 0.875rem;
}

.event-content.collapsed {
  max-height: 100px;
  overflow: hidden;
  cursor: pointer;
}

.event-content.collapsed:after {
  content: "▼ Click to expand";
  display: block;
  text-align: center;
  color: #666;
  margin-top: 0.5rem;
}

.event-content:not(.collapsed):after {
  content: "▲ Click to collapse";
  display: block;
  text-align: center;
  color: #666;
  margin-top: 0.5rem;
}

.summary-section {
  padding: 1rem;
  background: #E8F5E9;
  border-radius: 8px;
  margin-top: 1rem;
}

.summary-section h3 {
  margin-top: 0;
  color: #2E7D32;
}

.summary-section h4 {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
  color: #388E3C;
}

.summary-section code {
  background: #C8E6C9;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Monaco', 'Courier New', monospace;
}
```

## HTML Structure

Add to your existing `index.html`:

```html
<div id="chat-container">
  <div id="agent-events">
    <!-- Agent events will be appended here -->
  </div>
  
  <div id="task-summary">
    <!-- Task summary will be displayed here -->
  </div>
  
  <div id="chat-messages">
    <!-- Final response messages -->
  </div>
  
  <div id="chat-input">
    <textarea id="message-input" placeholder="Enter your task..."></textarea>
    <button onclick="sendMessage()">Send</button>
  </div>
</div>
```

## Usage

```javascript
// When user sends a message
function sendMessage() {
  const input = document.getElementById('message-input');
  const message = input.value.trim();
  
  if (!message) return;
  
  // Clear previous events
  document.getElementById('agent-events').innerHTML = '';
  document.getElementById('task-summary').innerHTML = '';
  
  // Show user message
  displayUserMessage(message);
  
  // Connect to event stream
  connectToAgentStream(message);
  
  input.value = '';
}
```

## Best Practices

1. **Progressive Enhancement** - Show a loading indicator before events arrive
2. **Error Handling** - Gracefully handle stream disconnections
3. **Auto-scroll** - Keep newest events visible
4. **Event Filtering** - Allow users to filter event types
5. **Collapsible Content** - Make large content (plans, diffs) collapsible
6. **Visual Indicators** - Use icons and colors to distinguish event types
7. **Timestamps** - Show relative or absolute times
8. **Accessibility** - Use ARIA labels and semantic HTML

## Testing

Test with curl:

```bash
curl -X POST http://localhost:3000/api/chat/stream \
  -H "Content-Type: application/json" \
  -d '{"message": "Create a simple hello world function"}' \
  --no-buffer
```

You should see output like:
```
data: session:abc-123-def

data: event:{"type":"agent_started","agent_name":"ProductOwner",...}

data: event:{"type":"agent_thought","thought":"I need to understand...",...}

data: event:{"type":"tool_call","tool_name":"read_structure",...}

data: data:Task completed successfully

data: done
```

## Next Steps

1. Implement the HTML/CSS/JavaScript
2. Test with various tasks
3. Add event filtering UI
4. Implement event persistence (optional)
5. Add replay functionality (optional)
