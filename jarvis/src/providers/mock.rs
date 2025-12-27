use crate::providers::LlmProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct MockLlm;

#[async_trait]
impl LlmProvider for MockLlm {
    async fn generate(&self, prompt: &str) -> Result<String> {
        if prompt.contains("Identity: ProductOwner") {
            Ok("HANDOFF RequirementsEngineer InitialPlanningComplete PO_Analyzed_Codebase".to_string())
        } else if prompt.contains("Identity: RequirementsEngineer") {
            Ok("HANDOFF SeniorDeveloper PlanGenerated 1.Implement_Login".to_string())
        } else if prompt.contains("Identity: SeniorDeveloper") {
            Ok("HANDOFF AccessibilityExpert ImplementationComplete Dev_Implemented_Login".to_string())
        } else if prompt.contains("Identity: AccessibilityExpert") {
            Ok("HANDOFF SEOExpert AccessibilityCheckPassed Accessibility_Verified".to_string())
        } else if prompt.contains("Identity: SEOExpert") {
            Ok("HANDOFF SecurityExpert SEOCheckPassed SEO_Verified".to_string())
        } else if prompt.contains("Identity: QATester") {
            Ok("HANDOFF Librarian QAVerificationPassed QA_Verified".to_string())
        } else if prompt.contains("Identity: SecurityExpert") {
            Ok("HANDOFF QATester SecurityCheckPassed Security_Verified".to_string())
        } else if prompt.contains("Identity: Librarian") {
            Ok("SUCCESS Task finalized by Librarian".to_string())
        } else {
            Ok("SUCCESS Default mock response".to_string())
        }
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }
}
