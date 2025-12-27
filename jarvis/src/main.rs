use anyhow::Result;
use tracing::info;
use jarvis::orchestration::Manager;
use jarvis::providers::ollama::OllamaProvider;
use jarvis::agents::planning::{ProductOwner, RequirementsEngineer};
use jarvis::agents::development::SeniorDeveloper;
use jarvis::agents::validation::QATester;
use jarvis::agents::security::SecurityExpert;
use jarvis::agents::documentation::Librarian;
use jarvis::agents::refinement::{AccessibilityExpert, SEOExpert};
use jarvis::tools::fs::{ListFilesTool, ReadFileTool, WriteFileTool, ApplyPatchTool, ReadStructureTool, SearchCodebaseTool};
use jarvis::tools::shell::{RunTestsTool, StaticAnalysisTool};
use jarvis::tools::git::{ReadDiffTool, GitCommitTool, GitCheckoutTool};
use jarvis::tools::memory::StorePreferenceTool;
use jarvis::tools::analysis::{AnalyzeDependenciesTool, FindCodeMarkersTool};
use jarvis::tools::project_cache::{CacheProjectStructureTool, GetCachedProjectStructureTool};
use jarvis::mcp::McpClient;
use jarvis::tools::mcp::McpTool;
use jarvis::providers::postgres::PostgresProvider;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::io::{self, Write};
use clap::{Parser, Subcommand};
use jarvis::config::Config;
use dialoguer::Input;

struct CliHitl;

impl jarvis::orchestration::HumanInTheLoop for CliHitl {
    fn consult(&self, agent_name: &str, task: &str, history: &[String]) -> Result<String> {
        println!("\n=== HUMAN INTERVENTION REQUIRED ===");
        println!("Agent: {}", agent_name);
        println!("Current Task/Context: {}", task);
        println!("--- History ---");
        for event in history {
            println!("- {}", event);
        }
        println!("====================================");
        print!("Please provide instructions for the agent: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The task to perform
    #[arg(short, long)]
    task: Option<String>,

    /// Ollama host
    #[arg(long)]
    ollama_host: Option<String>,

    /// Ollama port
    #[arg(long)]
    ollama_port: Option<u16>,

    /// Model to use
    #[arg(long)]
    model: Option<String>,

    /// Database URL for vector storage and persistence
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Session ID for continuing a conversation
    #[arg(long)]
    session_id: Option<String>,

    /// Path to MCP config file
    #[arg(long)]
    mcp_config: Option<String>,

    /// Start as an ACP server
    #[arg(long)]
    serve_acp: bool,

    /// Port for ACP server
    #[arg(long, default_value_t = 8000)]
    acp_port: u16,

    /// Start as an MCP server
    #[arg(long)]
    serve_mcp: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive setup of Jarvis configuration
    Setup,
}

async fn run_setup() -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    println!("Welcome to Jarvis Setup!");

    config.ollama_host = Input::new()
        .with_prompt("Ollama Host")
        .default(config.ollama_host)
        .interact_text()?;

    config.ollama_port = Input::new()
        .with_prompt("Ollama Port")
        .default(config.ollama_port)
        .interact_text()?;

    config.model = Input::new()
        .with_prompt("Model to use")
        .default(config.model)
        .interact_text()?;

    let db_url: String = Input::new()
        .with_prompt("Database URL (Postgres)")
        .with_initial_text(config.database_url.unwrap_or_default())
        .allow_empty(true)
        .interact_text()?;
    
    config.database_url = if db_url.is_empty() { None } else { Some(db_url) };

    let mcp_path: String = Input::new()
        .with_prompt("MCP Config Path")
        .with_initial_text(config.mcp_config.unwrap_or_default())
        .allow_empty(true)
        .interact_text()?;

    config.mcp_config = if mcp_path.is_empty() { None } else { Some(mcp_path) };

    config.save()?;
    println!("Configuration saved to {:?}", Config::get_config_path()?);
    Ok(())
}

async fn load_mcp_tools(config_path: &str) -> Result<Vec<Arc<dyn jarvis::tools::Tool>>> {
    let content = std::fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;
    let mut tools = Vec::new();

    if let Some(servers) = config["mcpServers"].as_object() {
        for (name, server_config) in servers {
            let command = server_config["command"].as_str().ok_or_else(|| anyhow::anyhow!("Missing command for MCP server {}", name))?;
            let args_val = server_config["args"].as_array();
            let mut args = Vec::new();
            if let Some(a) = args_val {
                for arg in a {
                    if let Some(s) = arg.as_str() {
                        args.push(s);
                    }
                }
            }

            info!("Spawning MCP server: {} ({} {:?})", name, command, args);
            let client = Arc::new(tokio::sync::Mutex::new(McpClient::spawn(command, &args).await?));
            
            let mut client_lock = client.lock().await;
            let mcp_tools = client_lock.list_tools().await?;
            for t in mcp_tools {
                tools.push(Arc::new(McpTool {
                    name: t.name,
                    description: t.description,
                    client: client.clone(),
                }) as Arc<dyn jarvis::tools::Tool>);
            }
        }
    }

    Ok(tools)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if let Some(Commands::Setup) = args.command {
        run_setup().await?;
        return Ok(());
    }

    let config = Config::load().unwrap_or_default();

    let ollama_host = args.ollama_host.unwrap_or(config.ollama_host);
    let ollama_port = args.ollama_port.unwrap_or(config.ollama_port);
    let model = args.model.unwrap_or(config.model);
    let database_url = args.database_url.or(config.database_url);
    let mcp_config = args.mcp_config.or(config.mcp_config);

    info!("Starting Jarvis AI Agent Squad...");

    let llm = Arc::new(OllamaProvider::new(ollama_host, ollama_port, model));
    
    let mut manager = Manager::new(3).with_hitl(Arc::new(CliHitl));

    let mut pg_provider_opt = None;
    if let Some(db_url) = database_url {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url).await?;
        
        let pg_provider = Arc::new(PostgresProvider::new(pool));
        pg_provider.setup().await?;
        
        manager = manager.with_vector_db(pg_provider.clone());
        manager = manager.with_persistence(pg_provider.clone());
        pg_provider_opt = Some(pg_provider);
    }

    let mut mcp_tools = Vec::new();
    if let Some(config_path) = mcp_config {
        mcp_tools = load_mcp_tools(&config_path).await?;
        info!("Loaded {} tools from MCP servers", mcp_tools.len());
    }

    let mut po_tools: Vec<Arc<dyn jarvis::tools::Tool>> = vec![
        Arc::new(ListFilesTool),
        Arc::new(ReadFileTool),
        Arc::new(ReadStructureTool),
        Arc::new(AnalyzeDependenciesTool),
        Arc::new(FindCodeMarkersTool),
    ];
    po_tools.extend(mcp_tools.clone());

    let mut dev_tools: Vec<Arc<dyn jarvis::tools::Tool>> = vec![
        Arc::new(WriteFileTool),
        Arc::new(ReadFileTool),
        Arc::new(ApplyPatchTool),
        Arc::new(GitCommitTool),
        Arc::new(GitCheckoutTool),
        Arc::new(AnalyzeDependenciesTool),
        Arc::new(FindCodeMarkersTool),
    ];
    dev_tools.extend(mcp_tools.clone());

    if let Some(pg_provider) = &pg_provider_opt {
        let search_tool = Arc::new(SearchCodebaseTool {
            llm: llm.clone(),
            vector_db: pg_provider.clone(),
        });
        let cache_tool = Arc::new(CacheProjectStructureTool {
            pg_provider: pg_provider.clone(),
        });
        let get_cache_tool = Arc::new(GetCachedProjectStructureTool {
            pg_provider: pg_provider.clone(),
        });
        
        po_tools.push(search_tool.clone());
        po_tools.push(cache_tool);
        po_tools.push(get_cache_tool);
        dev_tools.push(search_tool);
    }

    let po = Arc::new(ProductOwner::new(llm.clone(), po_tools));
    let re = Arc::new(RequirementsEngineer::new(llm.clone()));
    
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), dev_tools));

    let refinement_tools = vec![
        Arc::new(ReadDiffTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ApplyPatchTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(WriteFileTool) as Arc<dyn jarvis::tools::Tool>,
    ];
    let accessibility = Arc::new(AccessibilityExpert::new(llm.clone(), refinement_tools.clone()));
    let seo = Arc::new(SEOExpert::new(llm.clone(), refinement_tools));
    
    let qa_tools = vec![
        Arc::new(RunTestsTool) as Arc<dyn jarvis::tools::Tool>,
    ];
    let qa = Arc::new(QATester::new(llm.clone(), qa_tools));

    let security_tools = vec![
        Arc::new(StaticAnalysisTool) as Arc<dyn jarvis::tools::Tool>,
    ];
    let security = Arc::new(SecurityExpert::new(llm.clone(), security_tools));

    let mut lib_tools = vec![
        Arc::new(WriteFileTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ReadFileTool) as Arc<dyn jarvis::tools::Tool>,
    ];
    lib_tools.extend(mcp_tools);

    if let Some(pg_provider) = &pg_provider_opt {
        lib_tools.push(Arc::new(StorePreferenceTool {
            llm: llm.clone(),
            vector_db: pg_provider.clone(),
        }));
    }

    let librarian = Arc::new(Librarian::new(llm.clone(), lib_tools));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("AccessibilityExpert".to_string(), accessibility);
    manager.register_agent("SEOExpert".to_string(), seo);
    manager.register_agent("SecurityExpert".to_string(), security);
    manager.register_agent("QATester".to_string(), qa);
    manager.register_agent("Librarian".to_string(), librarian);

    let manager = Arc::new(manager);

    if args.serve_mcp {
        // Collect unique tools to expose via MCP
        let mut all_tools: Vec<Arc<dyn jarvis::tools::Tool>> = vec![
            Arc::new(ListFilesTool),
            Arc::new(ReadFileTool),
            Arc::new(WriteFileTool),
            Arc::new(ApplyPatchTool),
            Arc::new(ReadStructureTool),
            Arc::new(GitCommitTool),
            Arc::new(GitCheckoutTool),
            Arc::new(RunTestsTool),
            Arc::new(StaticAnalysisTool),
            Arc::new(AnalyzeDependenciesTool),
            Arc::new(FindCodeMarkersTool),
        ];
        if let Some(pg_provider) = &pg_provider_opt {
            all_tools.push(Arc::new(SearchCodebaseTool {
                llm: llm.clone(),
                vector_db: pg_provider.clone(),
            }));
        }
        
        let server = jarvis::mcp::McpServer::new(all_tools);
        server.run().await?;
    } else if args.serve_acp {
        info!("Starting ACP server on port {}...", args.acp_port);
        jarvis::orchestration::acp::start_acp_server(manager, args.acp_port).await?;
    } else if let Some(task) = args.task {
        let result = manager.run_with_session("ProductOwner", task, args.session_id).await?;
        println!("\n--- FINAL RESULT ---\n{}", result);
        
        // Print metrics summary
        println!("\n{}", manager.get_metrics_summary());
    } else {
        println!("No task provided. Use --task \"your task\" or run 'jarvis setup' to configure.");
    }

    Ok(())
}
