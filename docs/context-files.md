# Context Files Feature

## Overview

The context files feature allows you to pass specific files directly to agents, providing them with immediate access to relevant code without requiring them to search the filesystem. This significantly speeds up agent processing and improves accuracy by giving agents the exact context they need.

Context files can be provided via:
- CLI: `--context-files` flag with comma-separated paths
- GUI: File upload via drag-and-drop or file picker
- API: `context_files` array in POST requests

## Usage

### Basic Example

```bash
jarvis --task "Refactor the authentication logic" --context-files src/auth.rs
```

### Multiple Files

You can provide multiple files using comma-separated paths:

```bash
jarvis --task "Refactor the authentication logic" --context-files src/auth.rs,src/models.rs,src/utils.rs
```

### Combined with Session Resumption

```bash
jarvis --task "Continue refactoring" \
       --session-id abc123 \
       --context-files src/auth.rs,src/models.rs
```

## How It Works

1. **File Reading**: When you specify files via `--context-files`, Jarvis reads the content of each file at startup.
2. **Context Injection**: The file contents are injected into the agent's prompt under a "CONTEXT FILES" section.
3. **Agent Access**: All agents in the processing chain have access to these context files throughout the task execution.

## Benefits

- **Faster Processing**: Agents don't need to use tools to discover and read files
- **Better Focus**: Agents can concentrate on the specific files you want them to work with
- **Improved Accuracy**: Direct access to relevant code reduces errors from searching
- **Explicit Context**: You control exactly what context the agent sees

## Example Output

When context files are provided, they appear in the agent's prompt like this:

```
=== CONTEXT FILES ===
The following files have been provided as context for this task:

File: src/auth.rs
```
pub fn authenticate(token: &str) -> bool {
    // authentication logic
}
```

File: src/models.rs
```
pub struct User {
    pub id: u32,
    pub name: String,
}
```
======================
```

## Programmatic Usage

You can also use this feature programmatically:

```rust
use jarvis::orchestration::Manager;
use jarvis::agents::ContextFile;

let context_files = vec![
    ContextFile {
        path: "src/main.rs".to_string(),
        content: std::fs::read_to_string("src/main.rs")?,
    },
    ContextFile {
        path: "src/lib.rs".to_string(),
        content: std::fs::read_to_string("src/lib.rs")?,
    },
];

manager.run_with_session(
    "ProductOwner",
    "Review the code structure".to_string(),
    None,
    context_files,
).await?;
```

## GUI Mode Usage

In GUI mode, context files can be easily managed through the web interface:

1. **Attach Files**: Click the 📎 (paperclip) icon in the input area
2. **Select Files**: Choose one or more files from your file system
3. **Supported Types**: 
   - Source code: `.rs`, `.js`, `.ts`, `.tsx`, `.jsx`, `.py`, `.java`, `.go`, `.cpp`, `.c`, `.h`, `.hpp`, `.php`, `.rb`, `.swift`, `.kt`
   - Scripts: `.sh`, `.bash`, `.zsh`, `.fish`
   - Config: `.json`, `.yaml`, `.yml`, `.toml`, `.ini`, `.xml`, `.conf`
   - Documentation: `.md`, `.txt`, `.rst`, `.html`, `.css`, `.scss`
   - Other: `.sql`, `.graphql`, `.proto`, `.dockerfile`
4. **Visual Indicators**: Attached files appear as chips above the input box
5. **Remove Files**: Click the × button on any file chip to remove it
6. **Send Message**: Files are automatically included with your message

The GUI validates file types client-side and shows friendly error messages for unsupported formats.

## Tips

1. **Focus on Relevant Files**: Only include files that are directly relevant to the task to avoid overwhelming the context window
2. **File Size Considerations**: Be mindful of large files as they consume prompt space. The total context should stay within model limits
3. **Relative or Absolute Paths**: Both work, but relative paths from project root are recommended for consistency
4. **Update Context**: If files change during development, restart with updated context files or provide new files via GUI
5. **Combine with RAG**: Context files work alongside the vector database RAG system for maximum effectiveness
6. **GUI Advantage**: The GUI mode makes it easy to add/remove context files dynamically during a conversation
