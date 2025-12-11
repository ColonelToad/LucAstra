//! CLI commands for interactive LucAstra usage.

use clap::{Parser, Subcommand};
use lucastra_llm::{
    conversation::{Conversation, Message, Role},
    providers::{create_provider, CompletionRequest, EmbeddingRequest, ProviderConfig},
    rate_limit::RateLimiter,
};
use lucastra_search::vector::VectorIndex;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lucastra")]
#[command(about = "LucAstra AI-powered OS command line interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to provider config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive chat with LLM
    Chat {
        /// Initial message to start conversation
        message: Option<String>,

        /// Maximum conversation history to maintain
        #[arg(short, long, default_value = "10")]
        max_messages: usize,

        /// Enable streaming responses
        #[arg(short, long)]
        stream: bool,
    },

    /// Generate embeddings for text or files
    Embed {
        /// Text to embed (alternative to --file)
        #[arg(short, long)]
        text: Option<String>,

        /// File to embed
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Output embeddings to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Semantic search across indexed documents
    Search {
        /// Search query
        query: String,

        /// Number of results to return
        #[arg(short = 'n', long, default_value = "5")]
        top_k: usize,

        /// Minimum similarity score (0.0 - 1.0)
        #[arg(short, long, default_value = "0.0")]
        threshold: f32,

        /// Path to vector index
        #[arg(short, long)]
        index: Option<PathBuf>,
    },

    /// Index documents for semantic search
    Index {
        /// Directory or file to index
        path: PathBuf,

        /// Output index path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// File extensions to include (e.g., "txt,md,rs")
        #[arg(short, long)]
        extensions: Option<String>,
    },

    /// Show provider health and status
    Status {
        /// Show detailed provider information
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load provider config
    let config = if let Some(config_path) = cli.config {
        let config_str = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&config_str)?
    } else {
        ProviderConfig::default()
    };

    match cli.command {
        Commands::Chat {
            message,
            max_messages,
            stream,
        } => {
            chat_command(config, message, max_messages, stream).await?;
        }
        Commands::Embed {
            text,
            file,
            output,
        } => {
            embed_command(config, text, file, output).await?;
        }
        Commands::Search {
            query,
            top_k,
            threshold,
            index,
        } => {
            search_command(query, top_k, threshold, index).await?;
        }
        Commands::Index {
            path,
            output,
            extensions,
        } => {
            index_command(config, path, output, extensions).await?;
        }
        Commands::Status { verbose } => {
            status_command(config, verbose).await?;
        }
    }

    Ok(())
}

async fn chat_command(
    config: ProviderConfig,
    initial_message: Option<String>,
    _max_messages: usize,
    stream: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LucAstra Chat (provider: {})", config.provider);
    println!("Type 'exit' or 'quit' to end the conversation.\n");

    let provider = create_provider(config.clone()).await?;
    let mut conversation = Conversation::new(
        Some("You are LucAstra, a helpful AI assistant integrated into an augmented operating system.".to_string()),
    );
    let rate_limiter = RateLimiter::new(10); // 10 requests per minute

    // Send initial message if provided
    if let Some(msg) = initial_message {
        handle_user_message(&msg, &provider, &mut conversation, &rate_limiter, stream).await?;
    }

    // Interactive loop
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye! 👋");
            break;
        }

        handle_user_message(input, &provider, &mut conversation, &rate_limiter, stream).await?;

        // Trim conversation to max messages (TODO: implement proper trimming)
        // if conversation.messages().len() > max_messages {
        //     conversation.trim_to_message_count(max_messages);
        // }
    }

    Ok(())
}

async fn handle_user_message(
    message: &str,
    provider: &Box<dyn lucastra_llm::providers::LLMProvider>,
    conversation: &mut Conversation,
    rate_limiter: &RateLimiter,
    _stream: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    conversation.add_message(Message {
        role: Role::User,
        content: message.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    });

    // Rate limiting
    rate_limiter.acquire().await;

    // Generate completion
    let request = CompletionRequest {
        prompt: conversation.to_prompt(),
        max_tokens: Some(512),
        temperature: Some(0.7),
        ..Default::default()
    };

    let response = provider.complete(request).await?;

    println!("\n🤖 LucAstra: {}\n", response.content);

    conversation.add_message(Message {
        role: Role::Assistant,
        content: response.content,
        timestamp: chrono::Utc::now().timestamp(),
    });

    Ok(())
}

async fn embed_command(
    config: ProviderConfig,
    text: Option<String>,
    file: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_provider(config).await?;

    if !provider.supports_embeddings() {
        return Err(format!(
            "Provider '{}' does not support embeddings. Try using OpenAI provider.",
            provider.name()
        )
        .into());
    }

    let content = if let Some(text) = text {
        text
    } else if let Some(file_path) = file {
        std::fs::read_to_string(file_path)?
    } else {
        return Err("Either --text or --file must be provided".into());
    };

    let request = EmbeddingRequest {
        texts: vec![content],
        model: Some(provider.default_model().to_string()),
    };

    println!("📊 Generating embeddings...");
    let response = provider.embed(request).await?;

    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&response)?;
        std::fs::write(output_path, json)?;
        println!("✅ Embeddings saved");
    } else {
        println!("✅ Generated {} embeddings", response.embeddings.len());
        println!("   Dimensions: {}", response.dimensions);
        println!("   Model: {}", response.model);
    }

    Ok(())
}

async fn search_command(
    query: String,
    _top_k: usize,
    _threshold: f32,
    _index_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Load index from disk
    let _index = VectorIndex::new(); // Default dimension

    println!("🔍 Searching for: {}", query);
    println!("   (Index loading not yet implemented)");

    Ok(())
}

async fn index_command(
    _config: ProviderConfig,
    path: PathBuf,
    _output: Option<PathBuf>,
    _extensions: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 Indexing documents from: {}", path.display());
    println!("   (Document indexing not yet implemented)");

    Ok(())
}

async fn status_command(
    config: ProviderConfig,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏥 LucAstra Provider Status\n");

    let provider = create_provider(config.clone()).await?;

    println!("Provider: {}", provider.name());
    println!("Model: {}", provider.default_model());
    println!("Streaming: {}", if provider.supports_streaming() { "✓" } else { "✗" });
    println!("Embeddings: {}", if provider.supports_embeddings() { "✓" } else { "✗" });

    print!("Health: ");
    io::stdout().flush()?;

    match provider.health_check().await {
        Ok(true) => println!("✅ Online"),
        Ok(false) => println!("⚠️  Degraded"),
        Err(e) => println!("❌ Offline ({})", e),
    }

    if verbose {
        println!("\nConfiguration:");
        println!("{:#?}", config);
    }

    Ok(())
}
