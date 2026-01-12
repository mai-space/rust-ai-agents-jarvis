use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct MockLlm;

#[async_trait]
impl LlmProvider for MockLlm {
    async fn generate(&self, prompt: &str) -> Result<String> {
        // Extract agent name from prompt - works with both old and new prompt formats
        // Look for the agent identity which is always present
        let agent_name = if prompt.contains("ProductOwner:") || prompt.contains("ProductOwner ") {
            "ProductOwner"
        } else if prompt.contains("RequirementsEngineer:") || prompt.contains("RequirementsEngineer ") {
            "RequirementsEngineer"
        } else if prompt.contains("SeniorDeveloper:") || prompt.contains("SeniorDeveloper ") {
            "SeniorDeveloper"
        } else if prompt.contains("AccessibilityExpert:") || prompt.contains("AccessibilityExpert ") {
            "AccessibilityExpert"
        } else if prompt.contains("SEOExpert:") || prompt.contains("SEOExpert ") {
            "SEOExpert"
        } else if prompt.contains("SecurityExpert:") || prompt.contains("SecurityExpert ") {
            "SecurityExpert"
        } else if prompt.contains("QATester:") || prompt.contains("QATester ") {
            "QATester"
        } else if prompt.contains("Librarian:") || prompt.contains("Librarian ") {
            "Librarian"
        } else {
            ""
        };

        // Generate appropriate response based on agent
        match agent_name {
            "ProductOwner" => Ok("THOUGHT: I have analyzed the task.\nHANDOFF RequirementsEngineer InitialPlanningComplete PO_Analyzed_Codebase".to_string()),
            "RequirementsEngineer" => Ok("THOUGHT: I have created the detailed plan.\nHANDOFF SeniorDeveloper PlanGenerated 1.Implement_Login".to_string()),
            "SeniorDeveloper" => Ok("THOUGHT: I have implemented the code.\nHANDOFF AccessibilityExpert ImplementationComplete Dev_Implemented_Login".to_string()),
            "AccessibilityExpert" => Ok("THOUGHT: Accessibility check complete.\nHANDOFF SEOExpert AccessibilityCheckPassed Accessibility_Verified".to_string()),
            "SEOExpert" => Ok("THOUGHT: SEO check complete.\nHANDOFF SecurityExpert SEOCheckPassed SEO_Verified".to_string()),
            "SecurityExpert" => Ok("THOUGHT: Security check complete.\nHANDOFF QATester SecurityCheckPassed Security_Verified".to_string()),
            "QATester" => Ok("THOUGHT: QA verification complete.\nHANDOFF Librarian QAVerificationPassed QA_Verified".to_string()),
            "Librarian" => Ok("THOUGHT: Finalizing the task.\nSUCCESS Task finalized by Librarian".to_string()),
            _ => Ok("SUCCESS Default mock response".to_string()),
        }
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }
}
