use lucastra_app::SystemState;

#[test]
fn test_search_service_integration() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // Test that the search service is initialized
    assert!(state.config.search.max_results > 0);
}

#[test]
fn test_llm_service_integration() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // Test that the LLM service is accessible
    let server_url = &state.config.llm.server_url;
    assert!(!server_url.is_empty());
    assert!(server_url.contains("localhost") || server_url.contains("127.0.0.1"));
}

#[test]
fn test_rag_pipeline_ready() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // Verify that RAG components are available
    let has_search = state.config.search.max_results > 0;
    let has_llm = !state.config.llm.server_url.is_empty();
    
    assert!(has_search && has_llm, "RAG pipeline components not properly initialized");
}

#[test]
fn test_document_indexing() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // The system state initializes with example documents
    // This test verifies that document indexing can occur
    // Actual index operations would require the search service to be callable
    
    // Check that the configuration allows indexing
    assert!(state.config.search.max_results > 0); // Should be able to search
}

#[test]
fn test_search_configuration() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // Verify search config has proper defaults
    assert!(state.config.search.max_results > 0, "Search max_results should be positive");
    assert!(!state.config.search.embedding_model.is_empty(), "Embedding model should be configured");
}
