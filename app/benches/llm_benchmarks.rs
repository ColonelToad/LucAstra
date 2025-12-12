//! Performance benchmarks for LucAstra LLM and vector search.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lucastra_llm::{cache::EmbeddingCache, conversation::Conversation, rate_limit::RateLimiter};
use lucastra_search::vector::VectorIndex;
use std::time::Duration;
use tempfile::TempDir;

fn benchmark_vector_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_search");

    for size in [10, 100, 1000].iter() {
        let mut index = VectorIndex::new();

        // Populate index
        for i in 0..*size {
            let embedding = (0..384).map(|j| ((i + j) as f32) / 1000.0).collect();
            index.add_document(format!("doc_{}", i), embedding);
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let query = (0..384).map(|i| (i as f32) / 1000.0).collect();
            b.iter(|| black_box(index.search(&query, 5)));
        });
    }

    group.finish();
}

fn benchmark_cosine_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_similarity");

    for dim in [128, 384, 768, 1536].iter() {
        let vec_a: Vec<f32> = (0..*dim).map(|i| (i as f32) / 1000.0).collect();
        let vec_b: Vec<f32> = (0..*dim).map(|i| ((i + 1) as f32) / 1000.0).collect();

        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, _| {
            b.iter(|| black_box(lucastra_search::vector::cosine_similarity(&vec_a, &vec_b)));
        });
    }

    group.finish();
}

fn benchmark_embedding_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding_cache");

    let temp_dir = TempDir::new().unwrap();
    let mut cache = EmbeddingCache::new(temp_dir.path().to_path_buf());

    // Warm up cache
    for i in 0..100 {
        let embedding = (0..384).map(|j| ((i + j) as f32) / 1000.0).collect();
        cache.set(format!("text_{}", i), embedding);
    }

    group.bench_function("cache_get_hit", |b| {
        b.iter(|| black_box(cache.get("text_50")));
    });

    group.bench_function("cache_get_miss", |b| {
        b.iter(|| black_box(cache.get("text_9999")));
    });

    group.bench_function("cache_set", |b| {
        let embedding: Vec<f32> = (0..384).map(|i| (i as f32) / 1000.0).collect();
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            black_box(cache.set(format!("new_text_{}", counter), embedding.clone()))
        });
    });

    group.finish();
}

fn benchmark_conversation(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversation");

    group.bench_function("add_message", |b| {
        let mut conv = Conversation::new(Some("You are a helpful assistant.".to_string()));
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            black_box(conv.add_message(lucastra_llm::conversation::Message {
                role: lucastra_llm::conversation::Role::User,
                content: format!("Message {}", counter),
                timestamp: counter,
            }))
        });
    });

    group.bench_function("to_prompt_10_messages", |b| {
        let mut conv = Conversation::new(Some("You are a helpful assistant.".to_string()));
        for i in 0..10 {
            conv.add_message(lucastra_llm::conversation::Message {
                role: lucastra_llm::conversation::Role::User,
                content: format!("Message {}", i),
                timestamp: i,
            });
        }
        b.iter(|| black_box(conv.to_prompt()));
    });

    group.finish();
}

fn benchmark_rate_limiter(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiter");

    group.bench_function("acquire_under_limit", |b| {
        let limiter = RateLimiter::new(1000); // High limit
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.to_async(runtime)
            .iter(|| async { black_box(limiter.acquire().await) });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_vector_search,
    benchmark_cosine_similarity,
    benchmark_embedding_cache,
    benchmark_conversation,
    benchmark_rate_limiter
);
criterion_main!(benches);
