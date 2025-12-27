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
use jarvis::tools::fs::{ListFilesTool, ReadFileTool, WriteFileTool, ApplyPatchTool, ReadStructureTool};
use jarvis::tools::shell::{RunTestsTool, StaticAnalysisTool};
use jarvis::tools::git::ReadDiffTool;
use std::sync::Arc;
use std::io::{self, Write};
use clap::Parser;

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
    #[arg(short, long)]
    task: String,

    #[arg(long, default_value = "localhost")]
    ollama_host: String,

    #[arg(long, default_value_t = 11434)]
    ollama_port: u16,

    #[arg(long, default_value = "llama3")]
    model: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Starting Jarvis AI Agent Squad...");

    let llm = Arc::new(OllamaProvider::new(args.ollama_host, args.ollama_port, args.model));
    
    let mut manager = Manager::new(3).with_hitl(Arc::new(CliHitl));

    let po_tools = vec![
        Arc::new(ListFilesTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ReadFileTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ReadStructureTool) as Arc<dyn jarvis::tools::Tool>,
    ];

    let po = Arc::new(ProductOwner::new(llm.clone(), po_tools));
    let re = Arc::new(RequirementsEngineer::new(llm.clone()));
    
    let dev_tools = vec![
        Arc::new(WriteFileTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ReadFileTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ApplyPatchTool) as Arc<dyn jarvis::tools::Tool>,
    ];
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

    let lib_tools = vec![
        Arc::new(WriteFileTool) as Arc<dyn jarvis::tools::Tool>,
        Arc::new(ReadFileTool) as Arc<dyn jarvis::tools::Tool>,
    ];
    let librarian = Arc::new(Librarian::new(llm.clone(), lib_tools));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("AccessibilityExpert".to_string(), accessibility);
    manager.register_agent("SEOExpert".to_string(), seo);
    manager.register_agent("SecurityExpert".to_string(), security);
    manager.register_agent("QATester".to_string(), qa);
    manager.register_agent("Librarian".to_string(), librarian);

    let result = manager.run("ProductOwner", args.task).await?;

    println!("\n--- FINAL RESULT ---\n{}", result);

    Ok(())
}
