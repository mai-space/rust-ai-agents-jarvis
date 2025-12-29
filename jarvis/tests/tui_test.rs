use jarvis::orchestration::tui;
use jarvis::orchestration::Manager;
use jarvis::config::Config;
use jarvis::providers::mock::MockLlm;
use jarvis::agents::planning::{ProductOwner, RequirementsEngineer};
use jarvis::agents::development::SeniorDeveloper;
use jarvis::agents::refinement::{AccessibilityExpert, SEOExpert};
use jarvis::agents::validation::QATester;
use jarvis::agents::security::SecurityExpert;
use jarvis::agents::documentation::Librarian;
use std::sync::Arc;

// Helper function to register all agents needed for tests
fn register_all_agents(manager: &mut Manager, llm: Arc<MockLlm>) {
    let po = Arc::new(ProductOwner::new(llm.clone(), vec![]));
    let re = Arc::new(RequirementsEngineer::new(llm.clone(), vec![]));
    let dev = Arc::new(SeniorDeveloper::new(llm.clone(), vec![]));
    let accessibility = Arc::new(AccessibilityExpert::new(llm.clone(), vec![]));
    let seo = Arc::new(SEOExpert::new(llm.clone(), vec![]));
    let security = Arc::new(SecurityExpert::new(llm.clone(), vec![]));
    let qa = Arc::new(QATester::new(llm.clone(), vec![]));
    let lib = Arc::new(Librarian::new(llm.clone(), vec![]));

    manager.register_agent("ProductOwner".to_string(), po);
    manager.register_agent("RequirementsEngineer".to_string(), re);
    manager.register_agent("SeniorDeveloper".to_string(), dev);
    manager.register_agent("AccessibilityExpert".to_string(), accessibility);
    manager.register_agent("SEOExpert".to_string(), seo);
    manager.register_agent("SecurityExpert".to_string(), security);
    manager.register_agent("QATester".to_string(), qa);
    manager.register_agent("Librarian".to_string(), lib);
}

#[tokio::test]
async fn test_tui_module_exists() {
    // This test verifies that the TUI module is properly integrated
    let _config = Config::default();
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    register_all_agents(&mut manager, llm);
    let _manager = Arc::new(manager);

    // TUI functionality is tested by verifying the module can be accessed
    // We can't run the actual TUI in tests as it requires terminal interaction
    // but we verify that start_tui function exists and is callable
    
    // Verify the function exists and can be referenced
    // This ensures our public API is correct
    let _ = tui::start_tui;
    
    // Just verify types are compatible - don't actually call the function
    // as it requires an interactive terminal
    assert!(true, "TUI module is accessible");
}

#[test]
fn test_tui_dependencies() {
    // Verify TUI dependencies are available
    // This ensures ratatui and crossterm are properly linked
    
    // Check ratatui types
    let _backend_type: Option<ratatui::backend::CrosstermBackend<std::io::Stdout>> = None;
    let _terminal_type: Option<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>> = None;
    
    // Check crossterm types  
    let _event_type: Option<crossterm::event::Event> = None;
    let _keycode_type: Option<crossterm::event::KeyCode> = None;
    
    // If this compiles, dependencies are correctly configured
    assert!(true);
}

#[test]
fn test_config_for_tui() {
    // Verify Config can be used with TUI
    let config = Config::default();
    
    assert_eq!(config.ollama_host, "localhost");
    assert_eq!(config.ollama_port, 11434);
    assert!(!config.model.is_empty());
}

#[tokio::test]
async fn test_manager_with_tui() {
    // Verify Manager can be shared with TUI
    let llm = Arc::new(MockLlm);
    let mut manager = Manager::new(3);
    register_all_agents(&mut manager, llm);
    
    let manager = Arc::new(manager);
    
    // Verify manager can be cloned (required for TUI async operations)
    let _manager_clone = Arc::clone(&manager);
    
    // Verify manager can process tasks (core TUI functionality)
    let result = manager.run("ProductOwner", "test task".to_string()).await;
    assert!(result.is_ok());
}

#[test]
fn test_context_file_structure() {
    // Verify ContextFile structure used in TUI
    use jarvis::agents::ContextFile;
    
    let file = ContextFile {
        path: "test.rs".to_string(),
        content: "fn main() {}".to_string(),
    };
    
    assert_eq!(file.path, "test.rs");
    assert_eq!(file.content, "fn main() {}");
}
