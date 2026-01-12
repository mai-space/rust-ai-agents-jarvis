pub mod planning;
pub mod development;
pub mod validation;
pub mod security;
pub mod documentation;
pub mod refinement;

use anyhow::Result;
use async_trait::async_trait;
use crate::tools::Tool;
use crate::providers::{LlmProvider, VectorDbProvider};
use crate::project_context::ProjectMetadata;
use crate::events::{EventBroadcaster, TaskSummary};
use std::sync::Arc;
use serde_json::Value;
use tracing::{info, debug, warn};
use tokio::time::{timeout, Duration};

pub struct AgentContext {
    pub task: String,
    pub history: Vec<String>,
    pub vector_db: Option<Arc<dyn VectorDbProvider>>,
    pub available_agents: Vec<String>,
    pub project_metadata: Option<ProjectMetadata>,
    pub handoff_count: std::collections::HashMap<String, usize>,
    pub context_files: Vec<ContextFile>,
    pub event_broadcaster: Option<Arc<EventBroadcaster>>,
    pub task_summary: Arc<tokio::sync::RwLock<TaskSummary>>,
}

#[derive(Debug, Clone)]
pub struct ContextFile {
    pub path: String,
    pub content: String,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn identity(&self) -> String;
    fn capabilities(&self) -> Vec<Arc<dyn Tool>>;
    async fn process(&self, context: &mut AgentContext) -> Result<AgentOutput>;
}

#[derive(Debug)]
pub enum AgentOutput {
    Success(String),
    Handoff {
        target: String,
        reason: String,
        context: String,
    },
    Error(String),
}

pub async fn run_llm_agent(
    agent: &dyn Agent,
    llm: Arc<dyn LlmProvider>,
    context: &mut AgentContext,
) -> Result<AgentOutput> {
    let mut session_history = Vec::new();
    let mut executed_tools = std::collections::HashSet::new();
    let capabilities = agent.capabilities();
    let identity = agent.identity();
    
    let agent_name = identity.split(':').next().unwrap_or(&identity).to_string();

    info!("Agent starting: {}", &agent_name);
    
    // Emit agent started event
    if let Some(broadcaster) = &context.event_broadcaster {
        broadcaster.agent_started(agent_name.clone()).await;
    }
    
    // Track this agent in the summary
    {
        let mut summary = context.task_summary.write().await;
        summary.add_agent(agent_name.clone());
    }

    let tools_desc = capabilities.iter()
        .map(|t| format!("{}: {}", t.name(), t.description()))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = format!(
        "=== YOUR IDENTITY ===\n\
        {}\n\n\
        === AVAILABLE TOOLS ===\n\
        {}\n\n\
        === AVAILABLE AGENTS FOR HANDOFF ===\n\
        - {}\n\n\
        === COMMANDS YOU MUST USE ===\n\
        You MUST respond with EXACTLY ONE of these commands:\n\n\
        1. CALL <tool_name> <json_input>\n\
           Use this to execute a tool. JSON must be valid and on the SAME line.\n\
           Example: CALL read_file {{\"path\": \"README.md\"}}\n\n\
        2. HANDOFF <target_agent> <reason> <context_for_next_agent>\n\
           Use this to pass control to another agent. Must be all on ONE line.\n\
           Example: HANDOFF RequirementsEngineer needs_detailed_plan I've analyzed the codebase structure\n\n\
        3. SUCCESS <final_result>\n\
           Use this when your task is complete and successful.\n\
           Example: SUCCESS Task completed successfully. All files written.\n\n\
        4. ERROR <error_message>\n\
           Use this when you encounter an unrecoverable error.\n\
           Example: ERROR Cannot proceed - required file missing\n\n\
        5. PLAN <markdown_plan>\n\
           Use this to create a structured plan (for planning agents only).\n\
           Example: PLAN ## Overview\\nImplement authentication\\n## Steps\\n1. Create models\n\n\
        === CRITICAL RULES (MUST FOLLOW) ===\n\
        1. ALWAYS start with THOUGHT: on its own line to explain your reasoning\n\
        2. ALWAYS put your command on the NEXT line after THOUGHT:\n\
        3. Use REAL paths only (e.g., 'src/main.rs') - NEVER placeholders like '<file>' or '<path>'\n\
        4. If you don't know a path, use list_files or read_structure FIRST\n\
        5. NO conversational text - ONLY 'THOUGHT:' followed by ONE command\n\
        6. NO markdown code blocks (```) around commands - use plain text\n\
        7. ONE command per turn - no multiple commands\n\
        8. NEVER call the same tool with same arguments twice in one session\n\
        9. HANDOFF target MUST be from the available agents list above\n\
        10. NEVER HANDOFF to yourself - check your identity above\n\
        11. Stay in YOUR role - don't do other agents' work\n\
        12. Be decisive - 3-5 tool calls maximum, then HANDOFF or SUCCESS\n\
        13. If stuck or repeating, HANDOFF immediately\n\
        14. Check session history to avoid repeating actions\n\n\
        === CORRECT RESPONSE FORMAT ===\n\
        THOUGHT: [Your reasoning in one sentence]\n\
        [ONE COMMAND HERE]\n\n\
        === EXAMPLES ===\n\
        Example 1 - Using a tool:\n\
        THOUGHT: I need to see what files exist in the src directory.\n\
        CALL list_files {{\"path\": \"src\"}}\n\n\
        Example 2 - Reading a file:\n\
        THOUGHT: I should read the main.rs file to understand the current implementation.\n\
        CALL read_file {{\"path\": \"src/main.rs\"}}\n\n\
        Example 3 - Handing off:\n\
        THOUGHT: I've gathered enough information about the project structure.\n\
        HANDOFF RequirementsEngineer create_implementation_plan Project uses Rust with tokio async runtime\n\n\
        Example 4 - Completing successfully:\n\
        THOUGHT: All required changes have been implemented and verified.\n\
        SUCCESS Feature implementation complete. Created auth.rs and updated main.rs.\n\n\
        === WHAT NOT TO DO ===\n\
        ❌ WRONG: CALL read_file {{\"path\": \"<path_to_main_file>\"}}\n\
        ✅ RIGHT: CALL read_file {{\"path\": \"src/main.rs\"}}\n\n\
        ❌ WRONG: Let me think about this... I should probably...\n\
        ✅ RIGHT: THOUGHT: I should check the project structure.\n\n\
        ❌ WRONG: ```\\nCALL list_files {{\"path\": \".\"}}\\n```\n\
        ✅ RIGHT: CALL list_files {{\"path\": \".\"}}\n\n\
        ❌ WRONG: CALL tool1 {{...}} and then CALL tool2 {{...}}\n\
        ✅ RIGHT: CALL tool1 {{...}} [wait for result, then respond with next command]\n\n\
        Remember: Think step-by-step, be decisive, and follow the format EXACTLY.",
        identity, tools_desc, context.available_agents.join("\n- ")
    );

    let task_embeddings = if context.vector_db.is_some() {
        match timeout(Duration::from_secs(30), llm.get_embeddings(&context.task)).await {
            Ok(Ok(e)) => Some(e),
            Ok(Err(err)) => {
                warn!("Failed to get embeddings for task: {}", err);
                None
            }
            Err(_) => {
                warn!("Embedding generation timed out after 30 seconds");
                None
            }
        }
    } else {
        None
    };

    loop {
        let mut full_prompt = system_prompt.clone();

        // Add project context if available
        if let Some(project_meta) = &context.project_metadata {
            full_prompt.push_str("\n\n=== PROJECT CONTEXT ===\n");
            full_prompt.push_str(&project_meta.get_summary());
            full_prompt.push_str("======================\n");
        }

        if let (Some(vector_db), Some(embeddings)) = (&context.vector_db, &task_embeddings) {
            let mut combined_results = Vec::new();
            
            // Determine project_id for scoped search
            let project_id = context.project_metadata.as_ref()
                .map(|p| p.project_id.as_str())
                .unwrap_or("global");
            
            // 1. Project context (project-scoped)
            if let Ok(project_results) = vector_db.search_with_project(embeddings.clone(), 3, "project", project_id).await {
                for res in project_results {
                    combined_results.push(("Project", res));
                }
            }
            
            // 2. User preferences (global)
            if let Ok(user_results) = vector_db.search(embeddings.clone(), 3, "user").await {
                for res in user_results {
                    combined_results.push(("User Preference", res));
                }
            }

            if !combined_results.is_empty() {
                full_prompt.push_str("\n\nRelevant Context from Vector Database:\n");
                for (i, (source, res)) in combined_results.iter().enumerate() {
                    let res_str = res.to_string();
                    let display_res = if res_str.len() > 1000 {
                        format!("{}... (truncated)", &res_str[..1000])
                    } else {
                        res_str
                    };
                    full_prompt.push_str(&format!("{}. [{}] {}\n", i + 1, source, display_res));
                }
            }
        }

        // Add context files if provided
        if !context.context_files.is_empty() {
            full_prompt.push_str("\n\n=== CONTEXT FILES ===\n");
            full_prompt.push_str("The following files have been provided as context for this task:\n\n");
            for context_file in &context.context_files {
                full_prompt.push_str(&format!("File: {}\n", context_file.path));
                full_prompt.push_str("```\n");
                full_prompt.push_str(&context_file.content);
                full_prompt.push_str("\n```\n\n");
            }
            full_prompt.push_str("======================\n");
        }

        full_prompt.push_str(&format!("\n\nTask: {}\n", context.task));
        
        full_prompt.push_str("\nGlobal History:\n");
        if context.history.is_empty() {
            full_prompt.push_str("- No history yet.\n");
        } else {
            for h in &context.history {
                full_prompt.push_str(&format!("- {}\n", h));
            }
        }

        full_prompt.push_str("\nCurrent Session Trace (last 10 steps):\n");
        if session_history.is_empty() {
            full_prompt.push_str("- No steps taken in this session yet.\n");
        } else {
            let start = if session_history.len() > 10 { session_history.len() - 10 } else { 0 };
            for msg in &session_history[start..] {
                full_prompt.push_str(&format!("{}\n", msg));
            }
        }

        debug!("Calling LLM generate (prompt length: {})...", full_prompt.len());
        let response = match timeout(Duration::from_secs(180), llm.generate(&full_prompt)).await {
            Ok(res) => res?,
            Err(_) => {
                warn!("LLM generation timed out after 180 seconds");
                return Ok(AgentOutput::Error("LLM generation timed out. The prompt might be too large or the model too slow.".to_string()));
            }
        };
        debug!("LLM response received (length: {})", response.len());
        
        let cleaned_response = sanitize_model_response(&response);
        session_history.push(format!("Assistant: {}", cleaned_response));

        let trimmed_response = response.trim();
        let mut thought = String::new();
        let mut cmd_line = None;
        let mut placeholder_error = false;

        for line in trimmed_response.lines() {
            let mut line = line.trim();
            if line.is_empty() { continue; }

            // Strip markdown code blocks
            if line.starts_with("```") { continue; }

            // Strip common prefixes that models might hallucinate or echo
            if line.starts_with("Assistant:") { line = line["Assistant:".len()..].trim(); }
            if line.starts_with("Assistant") { line = line["Assistant".len()..].trim(); }
            if line.starts_with("System:") { line = line["System:".len()..].trim(); }
            if line.starts_with("Identity:") { line = line["Identity:".len()..].trim(); }
            if line.starts_with("Task:") { line = line["Task:".len()..].trim(); }
            if line.starts_with("THOUGHT:") { line = line["THOUGHT:".len()..].trim(); }
            if line.starts_with("Thought:") { line = line["Thought:".len()..].trim(); }
            if line.starts_with("Agent Thought:") { line = line["Agent Thought:".len()..].trim(); }
            if line.starts_with("Agent:") { line = line["Agent:".len()..].trim(); }
            if line.starts_with("Command:") { line = line["Command:".len()..].trim(); }

            // Strip bullets or markdown headers
            if line.starts_with("* ") { line = line["* ".len()..].trim(); }
            if line.starts_with("- ") { line = line["- ".len()..].trim(); }
            if line.starts_with("> ") { line = line["> ".len()..].trim(); }
            if line.starts_with("# ") { line = line["# ".len()..].trim(); }

            // Strip numbering like "1. " or "1) "
            if !line.is_empty() && line.chars().next().unwrap().is_ascii_digit() {
                if let Some(pos) = line.find(['.', ')']) {
                    if pos < 4 {
                        line = line[pos + 1..].trim();
                    }
                }
            }

            if line.is_empty() { continue; }

            if line.starts_with("CALL ") || line.starts_with("HANDOFF ") || line.starts_with("SUCCESS ") || line.starts_with("ERROR ") || line.starts_with("PLAN ") || line == "SUCCESS" || line == "ERROR" {
                if line.contains('<') && line.contains('>') {
                    session_history.push("System: Error: Detected descriptive placeholders like '<...>' in your command. You MUST use actual filesystem paths or values.".to_string());
                    placeholder_error = true;
                    cmd_line = None;
                    break;
                }
                cmd_line = Some(line.to_string());
                break;
            }
            
            if !thought.is_empty() { thought.push(' '); }
            thought.push_str(line);
        }

        if !thought.is_empty() {
            let display_thought = if thought.len() > 200 {
                format!("{}... (truncated)", &thought[..200])
            } else {
                thought.clone()
            };
            info!("Agent Thought: {}", display_thought);
            
            // Emit thought event
            if let Some(broadcaster) = &context.event_broadcaster {
                broadcaster.agent_thought(agent_name.clone(), thought).await;
            }
        }

        if let Some(line) = cmd_line {
            if line.starts_with("CALL") {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() < 3 {
                    session_history.push("System: Invalid CALL format. Use CALL <tool_name> <json_input>".to_string());
                    continue;
                }
                let tool_name = parts[1];
                let tool_input_str = parts[2];
                
                let tool = capabilities.iter().find(|t| t.name() == tool_name);
                match tool {
                    Some(t) => {
                        let input: Value = match serde_json::from_str(tool_input_str) {
                            Ok(v) => v,
                            Err(e) => {
                                session_history.push(format!("System: Error parsing JSON input: {}", e));
                                continue;
                            }
                        };
                        
                        let input_json = input.to_string();
                        if executed_tools.contains(&(tool_name.to_string(), input_json.clone())) {
                            session_history.push(format!("System: Information: You already called '{}' with these exact arguments in this session and have the result above. Do NOT repeat the call. Instead, use the information you gained to take the next step (e.g., read a specific file, or HANDOFF if you have enough info).", tool_name));
                            continue;
                        }
                        executed_tools.insert((tool_name.to_string(), input_json.clone()));

                        let input_summary = summarize_input(&input);
                        info!("Agent calling tool: {} with input: {}", tool_name, input_summary);
                        
                        // Emit tool call event
                        if let Some(broadcaster) = &context.event_broadcaster {
                            broadcaster.tool_call(agent_name.clone(), tool_name.to_string(), input_summary.clone()).await;
                        }
                        
                        match t.run(input).await {
                            Ok(res) => {
                                let output_summary = summarize_output(&res);
                                info!("Tool '{}' completed: {}", tool_name, output_summary);

                                // Emit tool result event
                                if let Some(broadcaster) = &context.event_broadcaster {
                                    broadcaster.tool_result(agent_name.clone(), tool_name.to_string(), output_summary, true).await;
                                }

                                let res_str = res.to_string();
                                let display_res = if res_str.len() > 2000 {
                                    format!("{}... (truncated, total {} chars)", &res_str[..2000], res_str.len())
                                } else {
                                    res_str
                                };
                                session_history.push(format!("System: Tool '{}' result: {}", tool_name, display_res))
                            },
                            Err(e) => {
                                // Emit tool error event
                                if let Some(broadcaster) = &context.event_broadcaster {
                                    broadcaster.tool_result(agent_name.clone(), tool_name.to_string(), format!("Error: {}", e), false).await;
                                }
                                session_history.push(format!("System: Tool '{}' error: {}", tool_name, e))
                            },
                        }
                    }
                    None => session_history.push(format!("System: Tool '{}' not found", tool_name)),
                }
            } else if line.starts_with("HANDOFF") {
                let parts: Vec<&str> = line.splitn(4, ' ').collect();
                if parts.len() < 4 {
                    session_history.push("System: Invalid HANDOFF format. Use HANDOFF <target> <reason> <context>".to_string());
                    continue;
                }
                let target = parts[1];
                
                // Prevent self-handoff
                let current_agent_name = identity.split(':').next().unwrap_or(&identity);
                if target == current_agent_name {
                    session_history.push(format!("System: Error: You CANNOT hand off to yourself ({}). You must hand off to a DIFFERENT agent. Review your task and either complete it (SUCCESS) or hand off to an appropriate different agent.", target));
                    continue;
                }
                
                if !context.available_agents.contains(&target.to_string()) {
                    session_history.push(format!("System: Error: Agent '{}' not found. Available agents are: {}. Please use one of the available agents.", target, context.available_agents.join(", ")));
                    continue;
                }
                
                // Track handoff patterns to detect loops
                *context.handoff_count.entry(target.to_string()).or_insert(0) += 1;
                
                return Ok(AgentOutput::Handoff {
                    target: target.to_string(),
                    reason: parts[2].to_string(),
                    context: parts[3].to_string(),
                });
            } else if line.starts_with("PLAN") {
                let plan = line.strip_prefix("PLAN ").unwrap_or("");
                if !plan.is_empty() {
                    info!("Agent created a plan");
                    
                    // Emit plan created event
                    if let Some(broadcaster) = &context.event_broadcaster {
                        broadcaster.plan_created(agent_name.clone(), plan.to_string()).await;
                    }
                    
                    session_history.push("System: Plan recorded. You can now proceed with execution or hand off to the appropriate agent.".to_string());
                } else {
                    session_history.push("System: PLAN command requires content. Use PLAN <your_markdown_plan>".to_string());
                }
            } else if line.starts_with("SUCCESS") {
                let result = line.strip_prefix("SUCCESS ").unwrap_or(&line);
                return Ok(AgentOutput::Success(result.to_string()));
            } else if line.starts_with("ERROR") {
                let err = line.strip_prefix("ERROR ").unwrap_or(&line);
                return Ok(AgentOutput::Error(err.to_string()));
            }
        } else if !placeholder_error {
            session_history.push("System: Unknown command format. Please use CALL, HANDOFF, SUCCESS, ERROR, or PLAN.".to_string());
        }

        // Prevent infinite loops in one agent process
        // Increased from 20 to 30 for agents with tools to have more flexibility
        let max_steps = if capabilities.is_empty() { 15 } else { 30 };
        if session_history.len() > max_steps {
            return Ok(AgentOutput::Error("Agent exceeded maximum interaction steps".to_string()));
        }
    }
}

fn summarize_input(input: &Value) -> String {
    if let Some(obj) = input.as_object() {
        let mut parts = Vec::new();
        for (k, v) in obj {
            if k == "content" || k == "patch" {
                let s = v.as_str().unwrap_or("");
                parts.push(format!("{}: <{} chars>", k, s.len()));
            } else {
                parts.push(format!("{}: {}", k, v));
            }
        }
        format!("{{ {} }}", parts.join(", "))
    } else {
        input.to_string()
    }
}

fn summarize_output(res: &Value) -> String {
    if let Some(obj) = res.as_object() {
        if let Some(files) = obj.get("files").and_then(|v| v.as_array()) {
            format!("{} files", files.len())
        } else if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
            format!("{} characters", content.len())
        } else if let Some(structure) = obj.get("structure").and_then(|v| v.as_array()) {
            format!("structure with {} top-level items", structure.len())
        } else if let Some(results) = obj.get("results").and_then(|v| v.as_array()) {
            format!("{} search results", results.len())
        } else if let Some(status) = obj.get("status") {
            format!("status: {}", status)
        } else {
            "success".to_string()
        }
    } else if let Some(arr) = res.as_array() {
        format!("array with {} items", arr.len())
    } else {
        "success".to_string()
    }
}

fn sanitize_model_response(response: &str) -> String {
    let mut cleaned_lines = Vec::new();
    for line in response.lines() {
        let mut line = line.trim();
        if line.is_empty() { continue; }

        // Skip lines that are just echoes of headers
        if line.starts_with("Identity:") || line.starts_with("Task:") || line.starts_with("Assistant:") || line.starts_with("System:") || line.starts_with("Agent:") || line.starts_with("Agent Thought:") || line.starts_with("Command:") {
            // Check if there is anything after the colon
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() > 1 && parts[1].trim().is_empty() {
                continue;
            }
            // Even if not empty, we might want to skip some specific headers completely
            if line.starts_with("Identity:") || line.starts_with("Task:") {
                continue;
            }
            
            // For Assistant:, Agent: and System:, we just strip the prefix
            if line.starts_with("Assistant:") { line = line["Assistant:".len()..].trim(); }
            if line.starts_with("Agent:") { line = line["Agent:".len()..].trim(); }
            if line.starts_with("Agent Thought:") { line = line["Agent Thought:".len()..].trim(); }
            if line.starts_with("System:") { line = line["System:".len()..].trim(); }
            if line.starts_with("Command:") { line = line["Command:".len()..].trim(); }
        }

        if line.starts_with("Assistant") { line = line["Assistant".len()..].trim(); }
        
        // Strip numbering like "1. " or "1) "
        if !line.is_empty() && line.chars().next().unwrap().is_ascii_digit() {
            if let Some(pos) = line.find(['.', ')']) {
                if pos < 4 {
                    line = line[pos + 1..].trim();
                }
            }
        }

        if !line.is_empty() {
            cleaned_lines.push(line.to_string());
        }
    }
    cleaned_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_summarize_input() {
        let input = json!({ "path": "test.txt", "content": "hello world" });
        assert_eq!(summarize_input(&input), "{ content: <11 chars>, path: \"test.txt\" }");

        let input = json!({ "query": "rust programming" });
        assert_eq!(summarize_input(&input), "{ query: \"rust programming\" }");
    }

    #[test]
    fn test_summarize_output() {
        let res = json!({ "files": ["a.rs", "b.rs"] });
        assert_eq!(summarize_output(&res), "2 files");

        let res = json!({ "content": "some long content" });
        assert_eq!(summarize_output(&res), "17 characters");

        let res = json!({ "status": "success" });
        assert_eq!(summarize_output(&res), "status: \"success\"");
        
        let res = json!({ "structure": [{"name": "src", "type": "directory"}] });
        assert_eq!(summarize_output(&res), "structure with 1 top-level items");
    }

    #[test]
    fn test_sanitize_model_response() {
        let input = "Assistant: 1. THOUGHT: Thinking\n2. CALL tool {}";
        let expected = "THOUGHT: Thinking\nCALL tool {}";
        assert_eq!(sanitize_model_response(input), expected);

        let input = "Identity: PO\nTask: Build\nSUCCESS Done";
        let expected = "SUCCESS Done";
        assert_eq!(sanitize_model_response(input), expected);

        let input = "Agent Thought: I will do X\nAgent: CALL tool {}\nCommand: SUCCESS Done";
        let expected = "I will do X\nCALL tool {}\nSUCCESS Done";
        assert_eq!(sanitize_model_response(input), expected);
    }

    #[tokio::test]
    async fn test_parser_robustness() -> Result<()> {
        struct MockLlm(String);
        #[async_trait::async_trait]
        impl crate::providers::LlmProvider for MockLlm {
            async fn generate(&self, _prompt: &str) -> Result<String> { Ok(self.0.clone()) }
            async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> { Ok(vec![]) }
        }

        struct DummyAgent;
        #[async_trait::async_trait]
        impl Agent for DummyAgent {
            fn identity(&self) -> String { "test".to_string() }
            fn capabilities(&self) -> Vec<Arc<dyn Tool>> { vec![] }
            async fn process(&self, _ctx: &mut AgentContext) -> Result<AgentOutput> { Ok(AgentOutput::Success("".to_string())) }
        }

        let mut context = AgentContext {
            task: "test".to_string(),
            history: vec![],
            vector_db: None,
            available_agents: vec!["test".to_string()],
            project_metadata: None,
            handoff_count: HashMap::new(),
            context_files: vec![],
            event_broadcaster: None,
            task_summary: Arc::new(tokio::sync::RwLock::new(crate::events::TaskSummary::new())),
        };

        // Test case 1: Assistant prefix and numbering
        let llm = Arc::new(MockLlm("Assistant: 1. THOUGHT: Thinking\n2. SUCCESS Done".to_string()));
        let out = run_llm_agent(&DummyAgent, llm, &mut context).await?;
        if let AgentOutput::Success(res) = out {
            assert_eq!(res, "Done");
        } else {
            panic!("Expected Success, got {:?}", out);
        }

        // Test case 2: Mixed echoes
        let llm = Arc::new(MockLlm("Identity: test\nTask: test\nTHOUGHT: Real thought\nSUCCESS Okay".to_string()));
        let out = run_llm_agent(&DummyAgent, llm, &mut context).await?;
        if let AgentOutput::Success(res) = out {
            assert_eq!(res, "Okay");
        } else {
            panic!("Expected Success, got {:?}", out);
        }

        // Test case 3: Command prefix
        let llm = Arc::new(MockLlm("Agent Thought: I'm done\nCommand: SUCCESS Finished".to_string()));
        let out = run_llm_agent(&DummyAgent, llm, &mut context).await?;
        if let AgentOutput::Success(res) = out {
            assert_eq!(res, "Finished");
        } else {
            panic!("Expected Success, got {:?}", out);
        }

        Ok(())
    }
}
