//! Example: Using the v1.1 async LLM providers with vector search and conversation management.
//!
//! This example demonstrates:
//! - Creating OpenAI and llamafile providers
//! - Generating embeddings for documents
//! - Performing semantic search with vector similarity
//! - Managing multi-turn conversations with context windows
//!
//! To run with OpenAI:
//! ```sh
//! export OPENAI_API_KEY=sk-...
//! cargo run --example v1_1_async_llm
//! ```

use lucastra_llm::{
    providers::{create_provider, ProviderConfig, EmbeddingRequest, CompletionRequest},
    Conversation, Message, Role,
};
use lucastra_search::vector::VectorIndex;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== LucAstra v1.1 Async LLM Demo ===\n");

    // Example 1: OpenAI Provider (requires OPENAI_API_KEY env var)
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        println!("1. Testing OpenAI Provider...");
        let config = ProviderConfig {
            provider: "openai".to_string(),
            api_key: Some(api_key),
            model: Some("gpt-4o-mini".to_string()),
            endpoint: None,
            temperature: Some(0.7),
            max_tokens: Some(256),
            timeout_secs: Some(30),
        };

        let provider = create_provider(config).await?;
        println!("   Provider: {}", provider.name());
        println!("   Model: {}", provider.default_model());
        println!("   Supports embeddings: {}", provider.supports_embeddings());

        // Health check
        match provider.health_check().await {
            Ok(true) => println!("   ✓ Health check passed\n"),
            Ok(false) => println!("   ✗ Health check failed\n"),
            Err(e) => println!("   ✗ Health check error: {}\n", e),
        }

        // Generate completion
        let request = CompletionRequest {
            prompt: "What is Rust programming language?".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            ..Default::default()
        };

        match provider.complete(request).await {
            Ok(response) => {
                println!("   Completion: {}", response.content);
                println!("   Tokens used: {:?}", response.tokens_used);
                println!("   Stop reason: {:?}\n", response.stop_reason);
            }
            Err(e) => println!("   Error: {}\n", e),
        }

        // Example 2: Generate embeddings and perform vector search
        println!("2. Testing Vector Search with Embeddings...");
        
        let documents = vec![
            ("Rust is a systems programming language focused on safety and performance.", "/docs/rust.txt"),
            ("Python is a high-level interpreted language known for simplicity.", "/docs/python.txt"),
            ("JavaScript is the language of the web, running in browsers.", "/docs/js.txt"),
            ("Go is a compiled language designed for concurrent systems.", "/docs/go.txt"),
        ];

        let mut index = VectorIndex::new();

        // Generate embeddings for documents
        let texts: Vec<String> = documents.iter().map(|(text, _)| text.to_string()).collect();
        let embed_request = EmbeddingRequest {
            texts: texts.clone(),
            model: None,
        };

        match provider.embed(embed_request).await {
            Ok(response) => {
                println!("   ✓ Generated {} embeddings (dim: {})", response.embeddings.len(), response.dimensions);
                
                // Index documents with their embeddings
                for ((text, path), embedding) in documents.iter().zip(response.embeddings.iter()) {
                    index.add_document(
                        PathBuf::from(path),
                        embedding.clone(),
                        text.to_string(),
                    )?;
                }

                // Perform semantic search
                let query = "What language is good for system programming?";
                println!("   Query: \"{}\"", query);
                
                let query_embed_request = EmbeddingRequest {
                    texts: vec![query.to_string()],
                    model: None,
                };

                let query_embedding = provider.embed(query_embed_request).await?;
                let results = index.search(&query_embedding.embeddings[0], 3)?;

                println!("   Top results:");
                for (i, result) in results.iter().enumerate() {
                    println!("     {}. [score: {:.3}] {}", i + 1, result.score, result.snippet);
                }
                println!();
            }
            Err(e) => println!("   Embeddings error: {}\n", e),
        }
    } else {
        println!("1. OpenAI Provider skipped (set OPENAI_API_KEY to test)\n");
    }

    // Example 3: Conversation Management
    println!("3. Testing Conversation Management...");
    
    let mut conversation = Conversation::new(Some(
        "You are a helpful programming assistant. Be concise.".to_string()
    ))
    .with_max_messages(10)
    .with_max_tokens(Some(4000));

    println!("   Conversation ID: {}", conversation.id);
    
    // Simulate multi-turn conversation
    conversation.add_user_message("What is Rust?".to_string());
    conversation.add_assistant_message(
        "Rust is a systems programming language focused on safety, speed, and concurrency.".to_string()
    );
    conversation.add_user_message("What about its memory management?".to_string());
    conversation.add_assistant_message(
        "Rust uses ownership and borrowing to manage memory without garbage collection.".to_string()
    );

    println!("   Messages in conversation: {}", conversation.len());
    println!("   \nConversation history:");
    for msg in conversation.messages() {
        let role = match msg.role {
            Role::System => "System",
            Role::User => "User",
            Role::Assistant => "Assistant",
        };
        println!("     {}: {}", role, msg.content);
    }
    println!();

    // Example 4: Llamafile Provider (local, no API key needed)
    println!("4. Testing Llamafile Provider...");
    let llamafile_config = ProviderConfig {
        provider: "llamafile".to_string(),
        api_key: None,
        endpoint: Some("http://localhost:8000".to_string()),
        model: None,
        temperature: Some(0.7),
        max_tokens: Some(256),
        timeout_secs: Some(30),
    };

    let llamafile = create_provider(llamafile_config).await?;
    println!("   Provider: {}", llamafile.name());
    
    match llamafile.health_check().await {
        Ok(true) => {
            println!("   ✓ Llamafile server is running");
            
            let request = CompletionRequest {
                prompt: "Explain async/await in Rust:".to_string(),
                max_tokens: Some(150),
                ..Default::default()
            };

            match llamafile.complete(request).await {
                Ok(response) => println!("   Response: {}", response.content),
                Err(e) => println!("   Error: {}", e),
            }
        }
        Ok(false) => println!("   ✗ Llamafile server not running (start with: llamafile -m model.gguf)"),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
