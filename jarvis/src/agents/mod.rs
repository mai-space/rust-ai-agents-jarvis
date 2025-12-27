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
use std::sync::Arc;
use serde_json::Value;
use tracing::{info, debug, warn};
use tokio::time::{timeout, Duration};

pub struct AgentContext {
    pub task: String,
    pub history: Vec<String>,
    pub vector_db: Option<Arc<dyn VectorDbProvider>>,
    pub available_agents: Vec<String>,
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

    info!("Agent starting: {}", identity.split(':').next().unwrap_or(&identity));

    let tools_desc = capabilities.iter()
        .map(|t| format!("{}: {}", t.name(), t.description()))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = format!(
        "Identity: {}\n\nAvailable Tools:\n{}\n\nAvailable Agents for HANDOFF:\n- {}\n\nCommands:\n- CALL <tool_name> {{ \"arg\": \"val\" }}\n- HANDOFF <target_agent> <reason> <context_for_next_agent>\n- SUCCESS <final_result>\n- ERROR <error_message>\n\n\
        Rules:\n\
        1. Provide a THOUGHT: line before your command to explain your reasoning.\n\
        2. Provide the command on a NEW line after your thought.\n\
        3. Use ONLY valid filesystem paths for tool arguments (e.g., '.', 'src/main.rs', 'README.md').\n\
        4. ABSOLUTELY PROHIBITED: Do NOT use descriptive placeholders like '<path_to_file>' or '<actual path>' in commands. If you don't know the path, use 'list_files' or 'read_structure' to find it.\n\
        5. Provide ONLY the THOUGHT and the command. Avoid conversational filler.\n\
        6. DO NOT repeat the same tool call with the same arguments if you have already received the result in this session.\n\
        7. DO NOT use markdown code blocks (```) for your commands. Provide them as plain text lines.\n\
        8. You MUST provide exactly ONE command (CALL, HANDOFF, SUCCESS, or ERROR) in every turn.\n\
        9. HANDOFF target MUST be one of the available agents listed above.\n\n\
        Example:\n\
        THOUGHT: I should list the files to see the project structure.\n\
        CALL list_files {{ \"path\": \".\" }}",
        identity, tools_desc, context.available_agents.join("\n- ")
    );

    let task_embeddings = if let Some(_) = &context.vector_db {
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

        if let (Some(vector_db), Some(embeddings)) = (&context.vector_db, &task_embeddings) {
            let mut combined_results = Vec::new();
            
            // 1. Project context
            if let Ok(project_results) = vector_db.search(embeddings.clone(), 3, "project").await {
                for res in project_results {
                    combined_results.push(("Project", res));
                }
            }
            
            // 2. User preferences
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

            // Strip bullets or markdown headers
            if line.starts_with("* ") { line = line["* ".len()..].trim(); }
            if line.starts_with("- ") { line = line["- ".len()..].trim(); }
            if line.starts_with("> ") { line = line["> ".len()..].trim(); }
            if line.starts_with("# ") { line = line["# ".len()..].trim(); }

            // Strip numbering like "1. " or "1) "
            if !line.is_empty() && line.chars().next().unwrap().is_ascii_digit() {
                if let Some(pos) = line.find(|c: char| c == '.' || c == ')') {
                    if pos < 4 {
                        line = line[pos + 1..].trim();
                    }
                }
            }

            if line.is_empty() { continue; }

            if line.starts_with("CALL ") || line.starts_with("HANDOFF ") || line.starts_with("SUCCESS ") || line.starts_with("ERROR ") || line == "SUCCESS" || line == "ERROR" {
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
                thought
            };
            info!("Agent Thought: {}", display_thought);
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
                        executed_tools.insert((tool_name.to_string(), input_json));

                        info!("Agent calling tool: {} with input: {}", tool_name, summarize_input(&input));
                        match t.run(input).await {
                            Ok(res) => {
                                info!("Tool '{}' completed: {}", tool_name, summarize_output(&res));

                                let res_str = res.to_string();
                                let display_res = if res_str.len() > 2000 {
                                    format!("{}... (truncated, total {} chars)", &res_str[..2000], res_str.len())
                                } else {
                                    res_str
                                };
                                session_history.push(format!("System: Tool '{}' result: {}", tool_name, display_res))
                            },
                            Err(e) => session_history.push(format!("System: Tool '{}' error: {}", tool_name, e)),
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
                if !context.available_agents.contains(&target.to_string()) {
                    session_history.push(format!("System: Error: Agent '{}' not found. Available agents are: {}. Please use one of the available agents.", target, context.available_agents.join(", ")));
                    continue;
                }
                return Ok(AgentOutput::Handoff {
                    target: target.to_string(),
                    reason: parts[2].to_string(),
                    context: parts[3].to_string(),
                });
            } else if line.starts_with("SUCCESS") {
                let result = line.strip_prefix("SUCCESS ").unwrap_or(&line);
                return Ok(AgentOutput::Success(result.to_string()));
            } else if line.starts_with("ERROR") {
                let err = line.strip_prefix("ERROR ").unwrap_or(&line);
                return Ok(AgentOutput::Error(err.to_string()));
            }
        } else if !placeholder_error {
            session_history.push("System: Unknown command format. Please use CALL, HANDOFF, SUCCESS, or ERROR.".to_string());
        }

        // Prevent infinite loops in one agent process
        if session_history.len() > 20 {
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
        if line.starts_with("Identity:") || line.starts_with("Task:") || line.starts_with("Assistant:") || line.starts_with("System:") || line.starts_with("Agent:") || line.starts_with("Agent Thought:") {
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
        }

        if line.starts_with("Assistant") { line = line["Assistant".len()..].trim(); }
        
        // Strip numbering like "1. " or "1) "
        if !line.is_empty() && line.chars().next().unwrap().is_ascii_digit() {
            if let Some(pos) = line.find(|c: char| c == '.' || c == ')') {
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

        let input = "Agent Thought: I will do X\nAgent: CALL tool {}";
        let expected = "I will do X\nCALL tool {}";
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

        Ok(())
    }
}
