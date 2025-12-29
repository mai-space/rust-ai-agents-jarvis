use crate::orchestration::Manager;
use crate::config::Config;
use crate::agents::ContextFile;
use crate::events::AgentEvent;
use anyhow::Result;
use chrono;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::sync::Arc;
use std::io;
use tokio::sync::mpsc;

#[derive(Clone)]
enum MessageType {
    User,
    Assistant,
    Thought,
    ToolCall,
    ToolResult,
    Plan,
    Handoff,
    Event,
}

#[derive(Clone)]
struct Message {
    message_type: MessageType,
    role: String,      // "user", "assistant", agent name, or "System"
    content: String,
    #[allow(dead_code)]
    timestamp: i64,
}

struct TuiState {
    messages: Vec<Message>,
    input: String,
    session_id: Option<String>,
    context_files: Vec<ContextFile>,
    selected_agent: String,
    available_agents: Vec<String>,
    mode: InputMode,
    scroll_offset: usize,
    status_message: String,
    cursor_position: usize,
    show_help: bool,
    is_processing: bool,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    SelectAgent,
    AddFile,
}

impl TuiState {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            session_id: None,
            context_files: Vec::new(),
            selected_agent: "ProductOwner".to_string(),
            available_agents: vec![
                "ProductOwner".to_string(),
                "RequirementsEngineer".to_string(),
                "SeniorDeveloper".to_string(),
                "AccessibilityExpert".to_string(),
                "SEOExpert".to_string(),
                "SecurityExpert".to_string(),
                "QATester".to_string(),
                "Librarian".to_string(),
            ],
            mode: InputMode::Normal,
            scroll_offset: 0,
            status_message: "Press 'i' to start typing, '?' for help".to_string(),
            cursor_position: 0,
            show_help: false,
            is_processing: false,
        }
    }

    fn add_message(&mut self, role: String, content: String) {
        let message_type = match role.as_str() {
            "user" => MessageType::User,
            "assistant" => MessageType::Assistant,
            _ => MessageType::Assistant, // Default to Assistant for backwards compatibility
        };
        
        self.messages.push(Message {
            message_type,
            role,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    fn add_event(&mut self, event: AgentEvent) {
        let (message_type, role, content) = match event {
            AgentEvent::AgentThought { agent_name, thought, .. } => {
                (MessageType::Thought, agent_name, thought)
            }
            AgentEvent::ToolCall { agent_name, tool_name, input_summary, .. } => {
                (MessageType::ToolCall, agent_name, format!("Calling tool: {} - {}", tool_name, input_summary))
            }
            AgentEvent::ToolResult { agent_name, tool_name, output_summary, success, .. } => {
                let status = if success { "✓" } else { "✗" };
                (MessageType::ToolResult, agent_name, format!("{} Tool result: {} - {}", status, tool_name, output_summary))
            }
            AgentEvent::PlanCreated { agent_name, plan, .. } => {
                (MessageType::Plan, agent_name, format!("Plan: {}", plan))
            }
            AgentEvent::Handoff { from_agent, to_agent, reason, .. } => {
                (MessageType::Handoff, from_agent.clone(), format!("Handing off to {} - {}", to_agent, reason))
            }
            AgentEvent::TaskCompleted { agent_name, result, .. } => {
                (MessageType::Event, agent_name, format!("✓ Task completed: {}", result))
            }
            AgentEvent::TaskFailed { agent_name, error, .. } => {
                (MessageType::Event, agent_name, format!("✗ Task failed: {}", error))
            }
            AgentEvent::AgentStarted { agent_name, .. } => {
                let msg = format!("{} started", agent_name);
                (MessageType::Event, agent_name, msg)
            }
            AgentEvent::FileOperation { operation, path, .. } => {
                let op_str = match operation {
                    crate::events::FileOpType::Created => "Created",
                    crate::events::FileOpType::Modified => "Modified",
                    crate::events::FileOpType::Deleted => "Deleted",
                    crate::events::FileOpType::Read => "Read",
                };
                (MessageType::Event, "System".to_string(), format!("{} file: {}", op_str, path))
            }
            AgentEvent::LoopDetected { agents, .. } => {
                (MessageType::Event, "System".to_string(), format!("⚠ Loop detected: {:?}", agents))
            }
            AgentEvent::HumanInterventionRequested { agent_name, reason, .. } => {
                (MessageType::Event, agent_name, format!("🤔 Human intervention requested: {}", reason))
            }
        };

        self.messages.push(Message {
            message_type,
            role,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    fn add_context_file(&mut self, path: String, content: String) {
        self.context_files.push(ContextFile { path, content });
    }
}

pub async fn start_tui(manager: Arc<Manager>, _config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState::new();
    
    // Create channel for async task results
    let (tx, mut rx) = mpsc::unbounded_channel::<Result<(String, Option<String>)>>();
    
    // Create channel for agent events
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentEvent>();
    
    // Subscribe to agent events
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        let mut event_stream = manager_clone.event_broadcaster.subscribe().await;
        loop {
            match event_stream.recv().await {
                Some(event) => {
                    if event_tx.send(event).is_err() {
                        break; // Channel closed, exit
                    }
                }
                None => break, // Stream ended
            }
        }
    });

    let res = run_tui(&mut terminal, &mut state, manager, tx, &mut rx, &mut event_rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

async fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut TuiState,
    manager: Arc<Manager>,
    tx: mpsc::UnboundedSender<Result<(String, Option<String>)>>,
    rx: &mut mpsc::UnboundedReceiver<Result<(String, Option<String>)>>,
    event_rx: &mut mpsc::UnboundedReceiver<AgentEvent>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, state))?;

        // Check for agent events (limit to 10 per iteration to avoid blocking)
        let mut event_count = 0;
        while event_count < 10 {
            match event_rx.try_recv() {
                Ok(event) => {
                    state.add_event(event);
                    event_count += 1;
                }
                Err(_) => break,
            }
        }

        // Check for async results
        if let Ok(result) = rx.try_recv() {
            state.is_processing = false;
            match result {
                Ok((response, session_id)) => {
                    state.add_message("assistant".to_string(), response);
                    if let Some(sid) = session_id {
                        state.session_id = Some(sid);
                    }
                    state.status_message = "Response received".to_string();
                }
                Err(e) => {
                    state.status_message = format!("Error: {}", e);
                }
            }
        }

        // Handle input with timeout for non-blocking
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match state.mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('i') => {
                            state.mode = InputMode::Editing;
                            state.status_message = "-- INSERT -- (Ctrl+D to send, Esc to cancel)".to_string();
                        }
                        KeyCode::Char('a') => {
                            state.mode = InputMode::SelectAgent;
                            state.status_message = "Select agent (↑/↓ to navigate, Enter to select)".to_string();
                        }
                        KeyCode::Char('f') => {
                            state.mode = InputMode::AddFile;
                            state.status_message = "Enter file path:".to_string();
                            state.input.clear();
                        }
                        KeyCode::Char('n') => {
                            // New chat
                            state.messages.clear();
                            state.context_files.clear();
                            state.session_id = None;
                            state.status_message = "Started new chat".to_string();
                        }
                        KeyCode::Char('c') => {
                            // Clear context files
                            state.context_files.clear();
                            state.status_message = "Cleared context files".to_string();
                        }
                        KeyCode::Char('?') => {
                            state.show_help = !state.show_help;
                        }
                        KeyCode::Up => {
                            if state.scroll_offset > 0 {
                                state.scroll_offset -= 1;
                            }
                        }
                        KeyCode::Down => {
                            state.scroll_offset += 1;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => {
                            state.mode = InputMode::Normal;
                            state.input.clear();
                            state.status_message = "Cancelled".to_string();
                        }
                        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'd' => {
                            if !state.input.is_empty() && !state.is_processing {
                                // Send message
                                let message = state.input.clone();
                                state.add_message("user".to_string(), message.clone());
                                state.input.clear();
                                state.mode = InputMode::Normal;
                                state.is_processing = true;
                                state.status_message = "Processing...".to_string();

                                // Clone what we need for the async task
                                let manager = Arc::clone(&manager);
                                let agent = state.selected_agent.clone();
                                let session_id = state.session_id.clone();
                                let context_files = state.context_files.clone();
                                let tx = tx.clone();

                                // Spawn async task
                                tokio::spawn(async move {
                                    let result = manager
                                        .run_with_session(&agent, message, session_id, context_files)
                                        .await;
                                    let _ = tx.send(result);
                                });
                            }
                        }
                        KeyCode::Char(c) => {
                            state.input.insert(state.cursor_position, c);
                            state.cursor_position += 1;
                        }
                        KeyCode::Backspace => {
                            if state.cursor_position > 0 {
                                state.input.remove(state.cursor_position - 1);
                                state.cursor_position -= 1;
                            }
                        }
                        KeyCode::Delete => {
                            if state.cursor_position < state.input.len() {
                                state.input.remove(state.cursor_position);
                            }
                        }
                        KeyCode::Left => {
                            if state.cursor_position > 0 {
                                state.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if state.cursor_position < state.input.len() {
                                state.cursor_position += 1;
                            }
                        }
                        KeyCode::Home => {
                            state.cursor_position = 0;
                        }
                        KeyCode::End => {
                            state.cursor_position = state.input.len();
                        }
                        _ => {}
                    },
                    InputMode::SelectAgent => {
                        let current_idx = state.available_agents
                            .iter()
                            .position(|a| a == &state.selected_agent)
                            .unwrap_or(0);
                        
                        match key.code {
                            KeyCode::Esc => {
                                state.mode = InputMode::Normal;
                                state.status_message = "Agent selection cancelled".to_string();
                            }
                            KeyCode::Up => {
                                if current_idx > 0 {
                                    state.selected_agent = state.available_agents[current_idx - 1].clone();
                                }
                            }
                            KeyCode::Down => {
                                if current_idx < state.available_agents.len() - 1 {
                                    state.selected_agent = state.available_agents[current_idx + 1].clone();
                                }
                            }
                            KeyCode::Enter => {
                                state.mode = InputMode::Normal;
                                state.status_message = format!("Selected agent: {}", state.selected_agent);
                            }
                            _ => {}
                        }
                    }
                    InputMode::AddFile => match key.code {
                        KeyCode::Esc => {
                            state.mode = InputMode::Normal;
                            state.input.clear();
                            state.status_message = "File add cancelled".to_string();
                        }
                        KeyCode::Enter => {
                            let path = state.input.trim().to_string();
                            if !path.is_empty() {
                                match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        state.add_context_file(path.clone(), content);
                                        state.status_message = format!("Added file: {}", path);
                                    }
                                    Err(e) => {
                                        state.status_message = format!("Error reading file: {}", e);
                                    }
                                }
                            }
                            state.input.clear();
                            state.mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            state.input.push(c);
                        }
                        KeyCode::Backspace => {
                            state.input.pop();
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, state: &TuiState) {
    if state.show_help {
        render_help(f);
        return;
    }

    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),      // Messages
            Constraint::Length(3),  // Context files
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    // Header
    let title = if state.is_processing {
        "Jarvis AI - TUI [Processing...]"
    } else {
        "Jarvis AI - TUI"
    };
    
    let header_text = vec![
        Line::from(vec![
            Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Agent: "),
            Span::styled(&state.selected_agent, Style::default().fg(Color::Green)),
            Span::raw(" | Session: "),
            Span::styled(
                state.session_id.as_deref().unwrap_or("None"),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];
    
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Blue)));
    f.render_widget(header, chunks[0]);

    // Messages
    let messages: Vec<ListItem> = state
        .messages
        .iter()
        .skip(state.scroll_offset)
        .map(|m| {
            let (style, prefix) = match m.message_type {
                MessageType::User => (Style::default().fg(Color::Cyan), "You: ".to_string()),
                MessageType::Assistant => (Style::default().fg(Color::Green), "AI: ".to_string()),
                MessageType::Thought => (Style::default().fg(Color::Yellow), format!("[{}] 💭 ", m.role)),
                MessageType::ToolCall => (Style::default().fg(Color::Magenta), format!("[{}] 🔧 ", m.role)),
                MessageType::ToolResult => (Style::default().fg(Color::Blue), format!("[{}] ", m.role)),
                MessageType::Plan => (Style::default().fg(Color::LightGreen), format!("[{}] 📋 ", m.role)),
                MessageType::Handoff => (Style::default().fg(Color::LightYellow), format!("[{}] ➡️ ", m.role)),
                MessageType::Event => (Style::default().fg(Color::Gray), format!("[{}] ", m.role)),
            };
            
            let content = format!("{}{}", prefix, m.content);
            
            ListItem::new(Text::from(content)).style(style)
        })
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Messages")
            .style(Style::default().fg(Color::White)));
    f.render_widget(messages_list, chunks[1]);

    // Context files
    let files_text = if state.context_files.is_empty() {
        "No context files (press 'f' to add)".to_string()
    } else {
        format!("Context files: {}", 
            state.context_files.iter()
                .map(|f| f.path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    
    let context_files = Paragraph::new(files_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Context Files")
            .style(Style::default().fg(Color::Yellow)));
    f.render_widget(context_files, chunks[2]);

    // Input
    let input_text = if state.mode == InputMode::Editing || state.mode == InputMode::AddFile {
        state.input.as_str()
    } else {
        "(press 'i' to type message)"
    };
    
    let input_style = if state.mode == InputMode::Editing || state.mode == InputMode::AddFile {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .style(Style::default().fg(Color::White)));
    f.render_widget(input, chunks[3]);

    // Set cursor position when editing
    if state.mode == InputMode::Editing || state.mode == InputMode::AddFile {
        // Clamp cursor position to prevent overflow
        let cursor_x = state.cursor_position.min(u16::MAX as usize - 2) as u16;
        f.set_cursor(
            chunks[3].x + cursor_x + 1,
            chunks[3].y + 1,
        );
    }

    // Status bar
    let status = Paragraph::new(state.status_message.as_str())
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(status, chunks[4]);
}

fn render_help(f: &mut Frame) {
    let help_text = vec![
        Line::from(Span::styled("=== Jarvis TUI Help ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  q       - Quit"),
        Line::from("  ?       - Toggle this help"),
        Line::from("  ↑/↓     - Scroll messages"),
        Line::from(""),
        Line::from(Span::styled("Chat:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  i       - Start typing a message"),
        Line::from("  Ctrl+D  - Send message (while editing)"),
        Line::from("  Esc     - Cancel editing"),
        Line::from("  n       - Start new chat"),
        Line::from(""),
        Line::from(Span::styled("Context Files:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  f       - Add context file"),
        Line::from("  c       - Clear all context files"),
        Line::from(""),
        Line::from(Span::styled("Agents:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  a       - Select agent"),
        Line::from(""),
        Line::from("Press '?' to close help"),
    ];

    let help = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .style(Style::default().fg(Color::Green)));

    let area = centered_rect(60, 80, f.size());
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
