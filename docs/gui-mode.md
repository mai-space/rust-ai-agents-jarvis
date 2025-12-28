# GUI Mode - Web Chat Interface

## Overview

Jarvis now includes a modern web-based chat interface inspired by OpenChat, providing an intuitive way to interact with the AI agent squad through your browser. The GUI mode supports all existing features including file context attachments, session management, and real-time agent responses.

## Features

- **Modern Chat Interface**: Clean, intuitive UI similar to popular chat applications
- **File Attachments**: Drag and drop or click to attach files for context
- **Session Management**: Continue conversations across multiple interactions
- **Real-time Updates**: See agent responses as they are generated
- **Context Files Support**: Full support for providing files to agents for better context
- **Responsive Design**: Works on desktop and mobile browsers

## Starting the GUI Server

To start Jarvis in GUI mode, use the `--serve-gui` flag:

```bash
jarvis --serve-gui
```

By default, the server will start on port 3000. You can specify a custom port:

```bash
jarvis --serve-gui --gui-port 8080
```

Once started, open your browser and navigate to:
```
http://localhost:3000
```

## Configuration

The GUI mode uses the same configuration as the CLI mode. Before starting, ensure you have:

1. **Ollama configured**: The GUI needs to connect to your Ollama instance
2. **Database (optional)**: For session persistence and memory features

You can configure these using the setup command:

```bash
jarvis setup
```

Or run the GUI with specific configuration:

```bash
jarvis --serve-gui \
    --ollama-host localhost \
    --ollama-port 11434 \
    --model llama3 \
    --database-url "postgres://user:pass@localhost/jarvis"
```

## Using the GUI

### Basic Chat

1. Type your message in the input box at the bottom
2. Press Enter or click "Send" to submit
3. Wait for the AI agent squad to process your request
4. View the response in the chat window

### Attaching Files for Context

The GUI supports attaching files to provide context to the agents, just like the `--context-files` CLI option:

1. Click the 📎 (paperclip) icon in the input area
2. Select one or more files from your computer
3. Supported file types:
   - Source code: `.rs`, `.js`, `.ts`, `.py`, `.java`, `.go`, `.cpp`, `.c`, `.h`, `.hpp`
   - Config files: `.json`, `.yaml`, `.yml`, `.toml`, `.txt`
   - Documentation: `.md`, `.html`, `.css`
4. The files appear as chips above the input box
5. Remove a file by clicking the × button on its chip
6. Send your message - the files will be included as context

### Session Management

- Each chat starts a new session automatically
- The session ID is displayed in the header once created
- Click "New Chat" to start a fresh conversation
- Sessions are persisted in the database (if configured)

### Keyboard Shortcuts

- **Enter**: Send message
- **Shift + Enter**: New line in message

## API Endpoints

The GUI server exposes several REST API endpoints:

### POST /api/chat

Send a message to the agent squad.

**Request:**
```json
{
  "message": "Your message here",
  "session_id": "optional-session-id",
  "context_files": [
    {
      "path": "src/main.rs",
      "content": "file content here"
    }
  ]
}
```

**Response:**
```json
{
  "message_id": "uuid",
  "session_id": "uuid",
  "response": "Agent response here",
  "timestamp": 1234567890
}
```

### GET /api/session/:session_id

Retrieve chat history for a session.

**Response:**
```json
{
  "session_id": "uuid",
  "messages": [
    {
      "id": "uuid",
      "role": "user",
      "content": "Message content",
      "timestamp": 1234567890
    }
  ]
}
```

### POST /api/upload

Upload files for context (multipart/form-data).

**Response:**
```json
[
  {
    "path": "filename.rs",
    "content": "file content"
  }
]
```

## Examples

### Example 1: Simple Task

1. Start the GUI: `jarvis --serve-gui`
2. Open http://localhost:3000
3. Type: "Create a simple HTTP server in Rust using Axum"
4. Press Enter
5. Watch as the agent squad analyzes the task and provides a solution

### Example 2: With File Context

1. Start the GUI: `jarvis --serve-gui`
2. Open http://localhost:3000
3. Click the 📎 icon and select `src/main.rs` and `Cargo.toml`
4. Type: "Refactor the main function to improve error handling"
5. Press Enter
6. The agents will use your files as context for better results

### Example 3: Custom Port

```bash
# Start on port 8080
jarvis --serve-gui --gui-port 8080

# Access at http://localhost:8080
```

## Troubleshooting

### GUI won't start

- **Check if port is in use**: Try a different port with `--gui-port`
- **Verify configuration**: Run `jarvis setup` to configure Ollama and database
- **Check Ollama**: Ensure Ollama is running and accessible

### Can't send messages

- **Check Ollama connection**: Verify Ollama is running on the configured host/port
- **Check database**: If using persistence, ensure the database is accessible
- **Browser console**: Open browser developer tools (F12) to check for errors

### File upload fails

- **File size**: Very large files may time out
- **File format**: Ensure the file is plain text (not binary)
- **Encoding**: Files must be UTF-8 encoded

## Architecture

The GUI mode consists of:

1. **Backend Server** (`src/orchestration/gui.rs`):
   - Axum web server
   - REST API endpoints
   - Integration with the Manager and agent system
   - Session state management

2. **Frontend** (`src/static/index.html`):
   - Single-page application (HTML/CSS/JavaScript)
   - Responsive chat interface
   - File upload handling
   - Real-time updates

## Comparison with CLI Mode

| Feature | CLI Mode | GUI Mode |
|---------|----------|----------|
| Interface | Terminal | Web Browser |
| File Context | `--context-files` flag | File upload UI |
| Session Resumption | `--session-id` flag | Automatic |
| Multiple Simultaneous Tasks | Terminal tabs | Browser tabs |
| Accessibility | Command line users | All users |
| Remote Access | SSH required | HTTP only |

## Security Considerations

- The GUI server binds to `0.0.0.0` by default, making it accessible from any network interface
- For production use, consider:
  - Adding authentication
  - Using HTTPS with TLS certificates
  - Binding to `127.0.0.1` for local-only access
  - Implementing rate limiting
  - Adding CORS restrictions

## Future Enhancements

Planned improvements for the GUI mode:

- [ ] Real-time streaming of agent responses
- [ ] Syntax highlighting for code in responses
- [ ] Markdown rendering
- [ ] Dark mode toggle
- [ ] File preview before upload
- [ ] Drag and drop file upload
- [ ] Multi-user support with authentication
- [ ] Chat history search
- [ ] Export conversations
- [ ] Agent selection UI
