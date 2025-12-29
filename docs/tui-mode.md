# TUI Mode - Terminal User Interface

## Overview

Jarvis includes a Terminal User Interface (TUI) that provides all the functionality of the GUI mode directly in your terminal. The TUI mode is perfect for users who prefer keyboard-driven interfaces, work over SSH, or want to use Jarvis without a web browser.

## Features

- **Interactive Chat Interface**: Full-featured chat interface within your terminal
- **Context Files Support**: Add files for context to help agents understand your code
- **Session Management**: Automatic session tracking and resumption
- **Agent Selection**: Choose which agent to start with
- **Real-time Updates**: See agent responses as they are generated
- **Keyboard Navigation**: Intuitive keyboard shortcuts for all operations
- **No Browser Required**: Works entirely in the terminal

## Starting the TUI

To start Jarvis in TUI mode, use the `--serve-tui` flag:

```bash
jarvis --serve-tui
```

The TUI will launch immediately in your current terminal session.

## Configuration

The TUI mode uses the same configuration as the CLI and GUI modes. Before starting, ensure you have:

1. **Ollama configured**: The TUI needs to connect to your Ollama instance
2. **Database (optional)**: For session persistence and memory features

You can configure these using the setup command:

```bash
jarvis setup
```

Or run the TUI with specific configuration:

```bash
jarvis --serve-tui \
    --ollama-host localhost \
    --ollama-port 11434 \
    --model llama3 \
    --database-url "postgres://user:pass@localhost/jarvis"
```

## Using the TUI

### Interface Layout

The TUI is divided into five sections:

1. **Header**: Shows the current agent and session ID
2. **Messages**: Displays the conversation history
3. **Context Files**: Shows attached context files
4. **Input**: Area for typing messages
5. **Status Bar**: Shows current mode and helpful tips

### Keyboard Shortcuts

#### Navigation
- **`q`** - Quit the TUI
- **`?`** - Toggle help screen
- **`↑`/`↓`** - Scroll through messages

#### Chat
- **`i`** - Start typing a message (enters INSERT mode)
- **`Ctrl+D`** - Send the message (while in INSERT mode)
- **`Esc`** - Cancel editing and return to NORMAL mode
- **`n`** - Start a new chat (clears messages and context)

#### Text Editing (in INSERT mode)
- **`Backspace`** - Delete character before cursor
- **`Delete`** - Delete character at cursor
- **`←`/`→`** - Move cursor left/right
- **`Home`** - Move cursor to start of line
- **`End`** - Move cursor to end of line

#### Context Files
- **`f`** - Add a context file (prompts for file path)
- **`c`** - Clear all context files

#### Agent Selection
- **`a`** - Open agent selection menu
- **`↑`/`↓`** - Navigate agent list (in selection mode)
- **`Enter`** - Confirm agent selection
- **`Esc`** - Cancel agent selection

### Basic Workflow

1. **Start the TUI**: `jarvis --serve-tui`
2. **Add context files (optional)**: Press `f`, enter file path, press Enter
3. **Select an agent (optional)**: Press `a`, use arrow keys, press Enter
4. **Start typing**: Press `i` to enter INSERT mode
5. **Type your message**: Enter your task or question
6. **Send the message**: Press `Ctrl+D`
7. **Wait for response**: The AI agent squad will process your request
8. **Continue conversation**: Repeat steps 4-7

## Examples

### Example 1: Simple Task

```bash
# Start the TUI
jarvis --serve-tui

# In the TUI:
# 1. Press 'i' to start typing
# 2. Type: "Create a simple HTTP server in Rust using Axum"
# 3. Press Ctrl+D to send
# 4. Wait for the agent response
```

### Example 2: With File Context

```bash
# Start the TUI
jarvis --serve-tui

# In the TUI:
# 1. Press 'f' to add a file
# 2. Type: src/main.rs
# 3. Press Enter
# 4. Press 'f' again to add another file
# 5. Type: Cargo.toml
# 6. Press Enter
# 7. Press 'i' to start typing
# 8. Type: "Refactor the main function to improve error handling"
# 9. Press Ctrl+D to send
```

### Example 3: Agent Selection

```bash
# Start the TUI
jarvis --serve-tui

# In the TUI:
# 1. Press 'a' to select an agent
# 2. Use ↑/↓ to navigate to "SecurityExpert"
# 3. Press Enter to select
# 4. Press 'i' to start typing
# 5. Type: "Review this codebase for security vulnerabilities"
# 6. Press Ctrl+D to send
```

## Session Management

- **Automatic Session Tracking**: Each conversation automatically gets a session ID
- **Session Display**: The session ID is shown in the header
- **Session Persistence**: With database configured, sessions are saved automatically
- **Resume Sessions**: Start a new TUI session and continue where you left off (session ID persists across TUI restarts when using the same database)

## Tips and Best Practices

### Effective Context Files

When adding context files:
- Start with the most relevant files first
- Include configuration files (Cargo.toml, package.json) for better understanding
- Don't add too many files at once - be selective

### Agent Selection

Choose the right agent for your task:
- **ProductOwner**: Understanding project structure and requirements
- **RequirementsEngineer**: Breaking down complex tasks into steps
- **SeniorDeveloper**: Implementing code changes
- **SecurityExpert**: Security reviews and vulnerability scanning
- **QATester**: Writing and running tests
- **AccessibilityExpert**: Accessibility improvements
- **SEOExpert**: SEO optimizations
- **Librarian**: Documentation updates

### Keyboard Navigation

- The TUI uses vim-like modes (NORMAL and INSERT)
- In NORMAL mode, single keys trigger actions
- In INSERT mode, keys type text
- Always press `Esc` to return to NORMAL mode

## Troubleshooting

### TUI won't start

- **Check dependencies**: Ensure your terminal supports ANSI colors and escape sequences
- **Terminal compatibility**: Works best with modern terminals (iTerm2, Alacritty, Windows Terminal)
- **Verify configuration**: Run `jarvis setup` to configure Ollama and database

### Can't see cursor

- The cursor is only visible in INSERT mode (press `i`)
- Try resizing your terminal window
- Check terminal emulator settings for cursor visibility

### Text rendering issues

- Ensure your terminal font supports Unicode characters
- Try a different terminal emulator
- Check that TERM environment variable is set correctly

### Agent not responding

- **Check Ollama**: Verify Ollama is running on the configured host/port
- **Check database**: If using persistence, ensure the database is accessible
- **View status**: The status bar shows "Processing..." when waiting for a response

### Terminal size too small

- Minimum recommended size: 80x24 characters
- Resize your terminal window for better experience
- Some elements may not display properly on very small terminals

## Comparison with CLI and GUI Modes

| Feature | CLI Mode | GUI Mode | TUI Mode |
|---------|----------|----------|----------|
| Interface | Command line | Web Browser | Terminal |
| Interactive | No | Yes | Yes |
| File Context | `--context-files` flag | File upload UI | Interactive file add |
| Session Management | `--session-id` flag | Automatic | Automatic |
| Agent Selection | Fixed at start | Dropdown | Interactive menu |
| Live Updates | No | Yes | Yes |
| Keyboard Shortcuts | N/A | Limited | Extensive |
| Remote Access | SSH | HTTP | SSH |
| Resource Usage | Minimal | Medium | Minimal |

## Advantages of TUI Mode

1. **Terminal Native**: Works entirely in your terminal, no browser needed
2. **SSH Friendly**: Perfect for remote development over SSH
3. **Keyboard Driven**: All operations accessible via keyboard shortcuts
4. **Lightweight**: Minimal resource usage compared to GUI
5. **Scriptable**: Can be integrated into terminal workflows
6. **No Port Conflicts**: Doesn't require opening a web port
7. **Familiar Interface**: Uses common terminal conventions (vim-like modes)

## Integration with Terminal Workflow

The TUI integrates seamlessly with your terminal workflow:

```bash
# Example: Using TUI in a development workflow
cd my-project

# Quick code review
jarvis --serve-tui
# Press 'f', add src/main.rs
# Press 'i', type "Review this code for improvements"
# Press Ctrl+D, review response, press 'q' to quit

# Continue with normal development
git status
```

## Future Enhancements

Potential improvements for TUI mode:

- [ ] Syntax highlighting for code in messages
- [ ] Split-screen view for comparing code
- [ ] Mouse support for clicking on elements
- [ ] Color themes and customization
- [ ] Message history search (search-as-you-type)
- [ ] Copy/paste integration with system clipboard
- [ ] Auto-completion for file paths
- [ ] Agent response streaming with progress indicator
- [ ] Export conversation to file
- [ ] Multi-session support (switch between conversations)

## Advanced Usage

### Custom Key Bindings

The TUI uses hardcoded key bindings. Future versions may support custom key binding configuration.

### Terminal Emulator Recommendations

Recommended terminal emulators for best TUI experience:

**macOS:**
- iTerm2 (recommended)
- Alacritty
- Kitty

**Linux:**
- Alacritty (recommended)
- Kitty
- GNOME Terminal
- Konsole

**Windows:**
- Windows Terminal (recommended)
- Alacritty
- ConEmu

### Scripting and Automation

While the TUI is interactive, you can still use CLI mode for automation:

```bash
# For automation, use CLI mode
jarvis --task "automated task" --context-files file1.rs,file2.rs

# For interactive work, use TUI mode
jarvis --serve-tui
```

## Conclusion

The TUI mode provides a powerful, keyboard-driven interface for interacting with Jarvis directly in your terminal. It combines the best aspects of CLI and GUI modes, offering an interactive experience without requiring a web browser. Whether you're working locally or remotely over SSH, the TUI mode gives you full access to Jarvis's capabilities in a lightweight, efficient interface.
