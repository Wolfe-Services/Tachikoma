# 073 - Backend Token Counting

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 073
**Status:** Planned
**Dependencies:** 051-backend-trait
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement token counting and estimation for all backends to support context tracking, cost estimation, and rate limit management.

---

## Acceptance Criteria

- [x] Token counting interface
- [x] Provider-specific tokenizers
- [x] Message token estimation
- [x] Cost calculation
- [x] Token budget management

---

## Implementation Details

### 1. Token Types (src/tokens/types.rs)

```rust
//! Token counting types.

use serde::{Deserialize, Serialize};

/// Token count for a request/response.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TokenCount {
    /// Input/prompt tokens.
    pub input: u32,
    /// Output/completion tokens.
    pub output: u32,
}

impl TokenCount {
    /// Create a new token count.
    pub fn new(input: u32, output: u32) -> Self {
        Self { input, output }
    }

    /// Total tokens.
    pub fn total(&self) -> u32 {
        self.input + self.output
    }

    /// Add another count.
    pub fn add(&mut self, other: TokenCount) {
        self.input += other.input;
        self.output += other.output;
    }
}

impl std::ops::Add for TokenCount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            input: self.input + other.input,
            output: self.output + other.output,
        }
    }
}

impl std::ops::AddAssign for TokenCount {
    fn add_assign(&mut self, other: Self) {
        self.input += other.input;
        self.output += other.output;
    }
}

/// Token pricing per million tokens.
#[derive(Debug, Clone, Copy)]
pub struct TokenPricing {
    /// Input price per million tokens (USD).
    pub input_per_million: f64,
    /// Output price per million tokens (USD).
    pub output_per_million: f64,
}

impl TokenPricing {
    /// Create pricing.
    pub fn new(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            input_per_million,
            output_per_million,
        }
    }

    /// Calculate cost for token count.
    pub fn calculate_cost(&self, tokens: TokenCount) -> f64 {
        let input_cost = tokens.input as f64 / 1_000_000.0 * self.input_per_million;
        let output_cost = tokens.output as f64 / 1_000_000.0 * self.output_per_million;
        input_cost + output_cost
    }

    /// Claude Opus 4 pricing.
    pub fn claude_opus() -> Self {
        Self::new(15.0, 75.0)
    }

    /// Claude Sonnet 4 pricing.
    pub fn claude_sonnet() -> Self {
        Self::new(3.0, 15.0)
    }

    /// Claude Haiku 3.5 pricing.
    pub fn claude_haiku() -> Self {
        Self::new(0.25, 1.25)
    }

    /// GPT-4o pricing.
    pub fn gpt4o() -> Self {
        Self::new(5.0, 15.0)
    }

    /// GPT-4o mini pricing.
    pub fn gpt4o_mini() -> Self {
        Self::new(0.15, 0.60)
    }

    /// Gemini 1.5 Pro pricing.
    pub fn gemini_pro() -> Self {
        Self::new(3.50, 10.50)
    }

    /// Free (local models).
    pub fn free() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Token budget for a session.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Maximum input tokens.
    pub max_input: u32,
    /// Maximum output tokens.
    pub max_output: u32,
    /// Maximum total tokens.
    pub max_total: u32,
    /// Maximum cost (USD).
    pub max_cost: Option<f64>,
}

impl TokenBudget {
    /// Create an unlimited budget.
    pub fn unlimited() -> Self {
        Self {
            max_input: u32::MAX,
            max_output: u32::MAX,
            max_total: u32::MAX,
            max_cost: None,
        }
    }

    /// Create a budget with limits.
    pub fn limited(max_input: u32, max_output: u32) -> Self {
        Self {
            max_input,
            max_output,
            max_total: max_input + max_output,
            max_cost: None,
        }
    }

    /// Add a cost limit.
    pub fn with_cost_limit(mut self, max_cost: f64) -> Self {
        self.max_cost = Some(max_cost);
        self
    }

    /// Check if usage is within budget.
    pub fn is_within(&self, usage: TokenCount, cost: f64) -> bool {
        if usage.input > self.max_input {
            return false;
        }
        if usage.output > self.max_output {
            return false;
        }
        if usage.total() > self.max_total {
            return false;
        }
        if let Some(max_cost) = self.max_cost {
            if cost > max_cost {
                return false;
            }
        }
        true
    }

    /// Get remaining budget.
    pub fn remaining(&self, usage: TokenCount) -> TokenCount {
        TokenCount {
            input: self.max_input.saturating_sub(usage.input),
            output: self.max_output.saturating_sub(usage.output),
        }
    }
}
```

### 2. Token Counter (src/tokens/counter.rs)

```rust
//! Token counting implementations.

use super::types::TokenCount;
use tachikoma_backends_core::{Message, MessageContent};

/// Trait for token counting.
pub trait TokenCounter: Send + Sync {
    /// Count tokens in text.
    fn count_text(&self, text: &str) -> u32;

    /// Count tokens in a message.
    fn count_message(&self, message: &Message) -> u32 {
        let content_tokens = match &message.content {
            MessageContent::Text(s) => self.count_text(s),
            MessageContent::Parts(parts) => {
                parts.iter().map(|p| {
                    match p {
                        tachikoma_backends_core::ContentPart::Text { text } => {
                            self.count_text(text)
                        }
                        tachikoma_backends_core::ContentPart::Image { .. } => {
                            // Images typically count as ~85 tokens for low detail
                            // or up to 765 for high detail
                            200
                        }
                    }
                }).sum()
            }
        };

        // Add overhead for message structure (~4 tokens)
        content_tokens + 4
    }

    /// Count tokens in multiple messages.
    fn count_messages(&self, messages: &[Message]) -> u32 {
        messages.iter().map(|m| self.count_message(m)).sum()
    }

    /// Estimate output tokens (rough heuristic).
    fn estimate_output(&self, prompt_tokens: u32) -> u32 {
        // Default: expect ~25% of prompt as output
        (prompt_tokens / 4).max(100)
    }
}

/// Simple character-based token counter (fallback).
#[derive(Debug, Clone, Default)]
pub struct SimpleTokenCounter {
    /// Characters per token estimate.
    chars_per_token: f32,
}

impl SimpleTokenCounter {
    /// Create with default ratio (4 chars/token).
    pub fn new() -> Self {
        Self {
            chars_per_token: 4.0,
        }
    }

    /// Create with custom ratio.
    pub fn with_ratio(chars_per_token: f32) -> Self {
        Self { chars_per_token }
    }
}

impl TokenCounter for SimpleTokenCounter {
    fn count_text(&self, text: &str) -> u32 {
        (text.len() as f32 / self.chars_per_token).ceil() as u32
    }
}

/// Claude tokenizer (approximation based on cl100k).
#[derive(Debug, Clone, Default)]
pub struct ClaudeTokenCounter;

impl TokenCounter for ClaudeTokenCounter {
    fn count_text(&self, text: &str) -> u32 {
        // Claude uses a similar tokenizer to cl100k_base
        // Approximate: 1 token per ~4 characters for English
        // More for non-English text
        let base = (text.len() as f32 / 4.0).ceil() as u32;

        // Adjust for whitespace and punctuation
        let whitespace_count = text.chars().filter(|c| c.is_whitespace()).count();
        let punct_count = text.chars().filter(|c| c.is_ascii_punctuation()).count();

        base + (whitespace_count / 4) as u32 + (punct_count / 2) as u32
    }
}

/// OpenAI tokenizer (approximation based on cl100k).
#[derive(Debug, Clone, Default)]
pub struct OpenAITokenCounter;

impl TokenCounter for OpenAITokenCounter {
    fn count_text(&self, text: &str) -> u32 {
        // Similar to Claude, uses cl100k_base
        (text.len() as f32 / 4.0).ceil() as u32
    }
}

/// Gemini tokenizer (approximation).
#[derive(Debug, Clone, Default)]
pub struct GeminiTokenCounter;

impl TokenCounter for GeminiTokenCounter {
    fn count_text(&self, text: &str) -> u32 {
        // Gemini uses SentencePiece, slightly different ratio
        (text.len() as f32 / 3.5).ceil() as u32
    }
}

/// Llama tokenizer (approximation).
#[derive(Debug, Clone, Default)]
pub struct LlamaTokenCounter;

impl TokenCounter for LlamaTokenCounter {
    fn count_text(&self, text: &str) -> u32 {
        // Llama uses SentencePiece with BPE
        (text.len() as f32 / 3.8).ceil() as u32
    }
}

/// Get the appropriate token counter for a provider.
pub fn counter_for_provider(provider: &str) -> Box<dyn TokenCounter> {
    match provider.to_lowercase().as_str() {
        "claude" | "anthropic" => Box::new(ClaudeTokenCounter),
        "openai" | "codex" | "gpt" => Box::new(OpenAITokenCounter),
        "gemini" | "google" => Box::new(GeminiTokenCounter),
        "ollama" | "llama" => Box::new(LlamaTokenCounter),
        _ => Box::new(SimpleTokenCounter::new()),
    }
}
```

### 3. Token Tracker (src/tokens/tracker.rs)

```rust
//! Token usage tracking.

use super::types::{TokenBudget, TokenCount, TokenPricing};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Tracks token usage over time.
#[derive(Debug)]
pub struct TokenTracker {
    /// Total tokens used.
    total: TokenCount,
    /// Tokens used this session.
    session: TokenCount,
    /// Token pricing.
    pricing: TokenPricing,
    /// Budget limits.
    budget: Option<TokenBudget>,
    /// History of usage.
    history: Vec<UsageRecord>,
}

/// Record of token usage.
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub timestamp: std::time::Instant,
    pub tokens: TokenCount,
    pub cost: f64,
    pub model: String,
}

impl TokenTracker {
    /// Create a new tracker.
    pub fn new(pricing: TokenPricing) -> Self {
        Self {
            total: TokenCount::default(),
            session: TokenCount::default(),
            pricing,
            budget: None,
            history: Vec::new(),
        }
    }

    /// Set a budget.
    pub fn with_budget(mut self, budget: TokenBudget) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Record token usage.
    pub fn record(&mut self, tokens: TokenCount, model: &str) {
        self.total += tokens;
        self.session += tokens;

        let cost = self.pricing.calculate_cost(tokens);

        self.history.push(UsageRecord {
            timestamp: std::time::Instant::now(),
            tokens,
            cost,
            model: model.to_string(),
        });

        debug!(
            input = tokens.input,
            output = tokens.output,
            cost = format!("${:.4}", cost),
            "Recorded token usage"
        );

        // Check budget
        if let Some(budget) = &self.budget {
            let total_cost = self.total_cost();
            if !budget.is_within(self.session, total_cost) {
                warn!(
                    session_tokens = self.session.total(),
                    total_cost = format!("${:.4}", total_cost),
                    "Approaching or exceeded budget"
                );
            }
        }
    }

    /// Get total usage.
    pub fn total(&self) -> TokenCount {
        self.total
    }

    /// Get session usage.
    pub fn session(&self) -> TokenCount {
        self.session
    }

    /// Get total cost.
    pub fn total_cost(&self) -> f64 {
        self.pricing.calculate_cost(self.total)
    }

    /// Get session cost.
    pub fn session_cost(&self) -> f64 {
        self.pricing.calculate_cost(self.session)
    }

    /// Get remaining budget.
    pub fn remaining_budget(&self) -> Option<TokenCount> {
        self.budget.as_ref().map(|b| b.remaining(self.session))
    }

    /// Check if within budget.
    pub fn is_within_budget(&self) -> bool {
        match &self.budget {
            Some(budget) => budget.is_within(self.session, self.total_cost()),
            None => true,
        }
    }

    /// Reset session counter.
    pub fn reset_session(&mut self) {
        self.session = TokenCount::default();
        info!("Session token counter reset");
    }

    /// Get usage history.
    pub fn history(&self) -> &[UsageRecord] {
        &self.history
    }

    /// Get summary statistics.
    pub fn summary(&self) -> UsageSummary {
        UsageSummary {
            total_tokens: self.total,
            session_tokens: self.session,
            total_cost: self.total_cost(),
            session_cost: self.session_cost(),
            request_count: self.history.len(),
        }
    }
}

/// Summary of token usage.
#[derive(Debug, Clone)]
pub struct UsageSummary {
    pub total_tokens: TokenCount,
    pub session_tokens: TokenCount,
    pub total_cost: f64,
    pub session_cost: f64,
    pub request_count: usize,
}

/// Thread-safe token tracker.
pub struct SharedTokenTracker {
    inner: Arc<RwLock<TokenTracker>>,
}

impl SharedTokenTracker {
    /// Create a new shared tracker.
    pub fn new(pricing: TokenPricing) -> Self {
        Self {
            inner: Arc::new(RwLock::new(TokenTracker::new(pricing))),
        }
    }

    /// Record usage.
    pub async fn record(&self, tokens: TokenCount, model: &str) {
        self.inner.write().await.record(tokens, model);
    }

    /// Get summary.
    pub async fn summary(&self) -> UsageSummary {
        self.inner.read().await.summary()
    }

    /// Check budget.
    pub async fn is_within_budget(&self) -> bool {
        self.inner.read().await.is_within_budget()
    }
}

impl Clone for SharedTokenTracker {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
```

### 4. Module Exports (src/tokens/mod.rs)

```rust
//! Token counting and tracking.

mod counter;
mod tracker;
mod types;

pub use counter::{
    counter_for_provider, ClaudeTokenCounter, GeminiTokenCounter, LlamaTokenCounter,
    OpenAITokenCounter, SimpleTokenCounter, TokenCounter,
};
pub use tracker::{SharedTokenTracker, TokenTracker, UsageRecord, UsageSummary};
pub use types::{TokenBudget, TokenCount, TokenPricing};
```

---

## Testing Requirements

1. Token counting is reasonably accurate
2. Message counting includes overhead
3. Cost calculation is correct
4. Budget tracking triggers warnings
5. Shared tracker is thread-safe

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Next: [074-backend-context.md](074-backend-context.md)
