# LucAstra v1.1 Roadmap - Async LLM & Vector Search

## Overview
Version 1.1 enhances LLM integration with async architecture, multi-provider support, and vector-based semantic search for improved RAG capabilities.

## Goals
1. **Async HTTP Client**: Replace blocking reqwest calls with proper async/await patterns
2. **Multi-Provider LLM Support**: Abstract provider interface supporting OpenAI, Anthropic, local llamafile
3. **Vector Embeddings**: Generate embeddings for documents and queries
4. **Vector Database**: Integrate HNSW or external vector store (Qdrant/Pinecone)
5. **Enhanced RAG**: Semantic search replaces keyword-based TF-IDF search
6. **Conversation Management**: Track multi-turn conversations with context windows

## Architecture Changes

### 1. Async HTTP Client (`llm/src/client.rs`)
**Current State**: Blocking calls with `Runtime::block_on()` wrapper
**Target State**: Native async with tokio runtime integration

```rust
// Current (v1.0)
pub fn complete(&self, prompt: &str) -> Result<String, ClientError> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(self.complete_async(prompt))
}

// Target (v1.1)
pub async fn complete(&self, prompt: &str) -> Result<String, ClientError> {
    let resp = self.client.post(&self.endpoint)
        .json(&req)
        .send()
        .await?;
    // ... streaming response handling
}
```

**Benefits**:
- Non-blocking I/O for concurrent requests
- Streaming responses for real-time output
- Better resource utilization

### 2. Provider Abstraction (`llm/src/providers/`)
**New Module Structure**:
```
llm/src/
├── providers/
│   ├── mod.rs              # Provider trait definition
│   ├── openai.rs           # OpenAI/Azure OpenAI
│   ├── anthropic.rs        # Claude API
│   ├── llamafile.rs        # Local llamafile (refactored from client.rs)
│   └── ollama.rs           # Ollama support
├── embeddings.rs           # Embedding generation
├── conversation.rs         # Multi-turn context management
└── streaming.rs            # Server-sent events (SSE) handling
```

**Provider Trait**:
```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> Result<impl Stream<Item = String>>;
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    fn supports_streaming(&self) -> bool;
    fn supports_embeddings(&self) -> bool;
}
```

### 3. Vector Search Integration (`search/` enhancement)
**Current**: TF-IDF keyword search
**Target**: Vector embeddings with HNSW index

**New Dependencies** (Cargo.toml):
```toml
[dependencies]
hnsw_rs = "0.3"  # Pure Rust HNSW implementation
ndarray = "0.15"  # Multi-dimensional arrays for vectors
```

**Search Module Changes**:
```rust
// search/src/vector.rs
pub struct VectorIndex {
    hnsw: Hnsw<f32, DistCosine>,
    doc_map: HashMap<usize, PathBuf>,
    embeddings: Vec<Vec<f32>>,
}

impl VectorIndex {
    pub async fn index_document(&mut self, path: PathBuf, content: &str, llm: &LLMService) {
        let embedding = llm.embed(vec![content.to_string()]).await?;
        self.hnsw.insert(embedding[0].clone(), self.doc_map.len());
        self.doc_map.insert(self.doc_map.len(), path);
    }

    pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<SearchResult> {
        self.hnsw.search(query_embedding, k, 30)
            .into_iter()
            .map(|(idx, score)| SearchResult {
                path: self.doc_map[&idx].clone(),
                score,
            })
            .collect()
    }
}
```

### 4. Conversation Context (`llm/src/conversation.rs`)
**Purpose**: Track multi-turn conversations with sliding window

```rust
pub struct Conversation {
    id: Uuid,
    messages: Vec<Message>,
    max_context_tokens: usize,
    system_prompt: Option<String>,
}

impl Conversation {
    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(Message { role, content, timestamp: Utc::now() });
        self.trim_to_window();
    }

    fn trim_to_window(&mut self) {
        // Keep only last N messages that fit within token limit
        // Tokenization via tiktoken-rs or simple word-based heuristic
    }

    pub fn to_prompt(&self, provider: &dyn LLMProvider) -> String {
        // Format messages according to provider's chat template
        // OpenAI: [{"role": "user", "content": "..."}]
        // Anthropic: Human: ... Assistant: ...
    }
}
```

## Implementation Plan

### Phase A: Async Refactor (Week 1)
- [ ] Refactor `LLMService` to use async methods
- [ ] Update `app/src/main.rs` to use Tokio runtime
- [ ] Replace blocking `health_check()` and `complete()` calls
- [ ] Add streaming response support with SSE parsing

### Phase B: Provider Abstraction (Week 1-2)
- [ ] Create `LLMProvider` trait in `llm/src/providers/mod.rs`
- [ ] Refactor llamafile client to implement trait
- [ ] Add OpenAI provider (`OPENAI_API_KEY` from env)
- [ ] Add Anthropic provider (`ANTHROPIC_API_KEY` from env)
- [ ] Add provider selection to config schema:
  ```json
  "llm": {
    "provider": "openai",  // or "anthropic", "llamafile"
    "api_key": "${OPENAI_API_KEY}",
    "model": "gpt-4o-mini",
    "endpoint": null  // optional for self-hosted
  }
  ```

### Phase C: Vector Embeddings (Week 2)
- [ ] Add `embed()` method to `LLMProvider` trait
- [ ] Implement embedding generation for OpenAI (`text-embedding-3-small`)
- [ ] Implement embedding generation for llamafile (if supported)
- [ ] Add fallback to local embedding models (e.g., `fastembed-rs`)
- [ ] Cache embeddings to avoid redundant API calls

### Phase D: Vector Search (Week 2-3)
- [ ] Add `hnsw_rs` and `ndarray` dependencies
- [ ] Create `search/src/vector.rs` with `VectorIndex`
- [ ] Migrate existing document indexing to use embeddings
- [ ] Replace TF-IDF search with vector similarity search
- [ ] Add hybrid search (keyword + semantic) with score fusion

### Phase E: Conversation Management (Week 3)
- [ ] Create `llm/src/conversation.rs` with context windowing
- [ ] Add conversation persistence (SQLite or JSON files)
- [ ] Add conversation API to `LLMService`:
  - `start_conversation(system_prompt)`
  - `send_message(conv_id, content)`
  - `get_history(conv_id)`
- [ ] Integrate into CLI/GUI for multi-turn interactions

### Phase F: Testing & Documentation (Week 3-4)
- [ ] Add integration tests for each provider
- [ ] Add benchmarks for vector search performance
- [ ] Document provider setup (API keys, endpoints)
- [ ] Add examples: RAG pipeline, streaming chat, multi-provider switching
- [ ] Update README with v1.1 features

## Configuration Schema Changes

**config/schema.json** (additions):
```json
{
  "llm": {
    "provider": "openai",
    "api_key": "${OPENAI_API_KEY}",
    "model": "gpt-4o-mini",
    "endpoint": null,
    "temperature": 0.7,
    "max_tokens": 4096,
    "timeout_secs": 30,
    "retry_attempts": 3
  },
  "embeddings": {
    "provider": "openai",
    "model": "text-embedding-3-small",
    "batch_size": 100,
    "cache_dir": "~/.lucastra/embeddings"
  },
  "vector_search": {
    "enabled": true,
    "index_path": "~/.lucastra/vector.index",
    "top_k": 5,
    "ef_construction": 200,
    "m": 16
  },
  "conversation": {
    "max_context_tokens": 8000,
    "persist": true,
    "persist_path": "~/.lucastra/conversations"
  }
}
```

## Testing Strategy

1. **Unit Tests**:
   - Provider trait implementations (mock responses)
   - Vector index operations (add, search, delete)
   - Conversation context trimming

2. **Integration Tests**:
   - E2E RAG pipeline: index → embed → search → infer
   - Multi-turn conversation with context
   - Provider switching mid-conversation

3. **Performance Tests**:
   - Vector search latency (target: <10ms for 10k docs)
   - Embedding generation throughput
   - Concurrent request handling

4. **Manual Tests**:
   - OpenAI API integration (requires key)
   - Anthropic API integration (requires key)
   - Llamafile streaming responses

## Success Metrics

- **Async Performance**: 10x improvement in concurrent request throughput
- **Search Quality**: >80% semantic relevance vs keyword-based baseline
- **Provider Support**: At least 3 providers (OpenAI, Anthropic, llamafile) working
- **Response Streaming**: <100ms latency to first token for streaming
- **Test Coverage**: 80%+ coverage for new modules

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| API rate limits | High costs, blocked requests | Add rate limiting, caching, local fallback |
| Embedding dimensionality mismatch | Search failures | Normalize dimensions, provider-specific configs |
| Large vector index memory usage | OOM for large corpora | Add index sharding, disk-backed storage |
| Breaking changes to v1.0 API | User disruption | Maintain backward compat layer, deprecation notices |

## Future Enhancements (v1.2+)

- **Fine-tuning**: Fine-tune local models on LucAstra-specific tasks
- **Multi-modal**: Support image/audio embeddings (CLIP, Whisper)
- **Agent Framework**: Tool calling, function execution, workflow orchestration
- **Distributed Search**: Shard vector index across nodes
- **Quantization**: 4-bit/8-bit quantized local models for reduced memory

## Migration Path from v1.0

Users on v1.0 can upgrade to v1.1 with minimal changes:

1. **Config Migration**: Add `llm.provider` field (defaults to "llamafile" for backward compat)
2. **API Changes**: Blocking calls deprecated but still supported via compatibility layer
3. **Data Migration**: Existing TF-IDF index auto-converts to hybrid mode (keyword + vector)

Example migration:
```rust
// v1.0
let service = LLMService::new("http://localhost:8000".to_string());
let response = service.infer(request)?;

// v1.1 (async, but v1.0 API still works)
let service = LLMService::from_config(&config).await?;
let response = service.infer(request).await?;
```

## Timeline

- **Week 1**: Async refactor + provider abstraction
- **Week 2**: Vector embeddings + search
- **Week 3**: Conversation management + integration
- **Week 4**: Testing, docs, release prep

**Target Release**: v1.1.0 (4 weeks from start)
