use crate::{Result, ToolError, ToolResult};
use lucastra_search::SearchService;
use tracing::info;

/// Search tool implementation
pub struct SearchTool<'a> {
    search_service: &'a SearchService,
}

impl<'a> SearchTool<'a> {
    pub fn new(search_service: &'a SearchService) -> Self {
        Self { search_service }
    }
    
    pub fn execute(&self, query: &str, top_k: usize) -> Result<ToolResult> {
        info!("Executing search tool: query='{}', top_k={}", query, top_k);
        
        let results = self.search_service.search(query, top_k)?;
        
        let output = if results.is_empty() {
            "No results found.".to_string()
        } else {
            results
                .iter()
                .map(|r| format!("{} (score: {:.2}): {}", r.path, r.score, r.snippet))
                .collect::<Vec<_>>()
                .join("\n")
        };
        
        Ok(ToolResult::success("search", output))
    }
}
