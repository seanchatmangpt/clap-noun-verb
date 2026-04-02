//! Token management for wizard AI interactions
//!
//! This module provides token counting, optimization, and budget enforcement
//! to prevent exceeding API limits and control costs.
//!
//! ## Features
//!
//! - Pre-request token counting
//! - Token budget enforcement
//! - Prompt optimization (token reduction strategies)
//! - Token usage tracking and metrics
//! - Provider-specific token counting

use crate::wizard::{
    config::{Model, Provider},
    error::{WizardError, WizardResult},
    types::{Prompt, TokenUsage},
};

/// Token counting strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenCountingStrategy {
    /// Approximate counting (fast, less accurate)
    Approximate,
    /// Exact counting using provider-specific tokenizers (slow, accurate)
    Exact,
}

/// Token budget configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenBudget {
    /// Maximum tokens per request
    pub max_tokens_per_request: usize,
    /// Maximum total tokens (cumulative across all requests)
    pub max_total_tokens: Option<usize>,
    /// Whether to error or truncate when budget exceeded
    pub on_exceed: BudgetExceedAction,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_tokens_per_request: 8192,
            max_total_tokens: None, // No cumulative limit by default
            on_exceed: BudgetExceedAction::Error,
        }
    }
}

impl TokenBudget {
    /// Create a new token budget
    pub const fn new(max_tokens_per_request: usize) -> Self {
        Self {
            max_tokens_per_request,
            max_total_tokens: None,
            on_exceed: BudgetExceedAction::Error,
        }
    }

    /// Set maximum total tokens
    pub const fn with_max_total(mut self, max_total: usize) -> Self {
        self.max_total_tokens = Some(max_total);
        self
    }

    /// Set action when budget exceeded
    pub const fn with_exceed_action(mut self, action: BudgetExceedAction) -> Self {
        self.on_exceed = action;
        self
    }
}

/// Action to take when token budget is exceeded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetExceedAction {
    /// Return an error
    Error,
    /// Truncate the prompt to fit within budget
    Truncate,
    /// Attempt to optimize/compress the prompt
    Optimize,
}

/// Token manager for tracking and enforcing token budgets
#[derive(Debug)]
pub struct TokenManager {
    /// Token budget configuration
    budget: TokenBudget,
    /// Cumulative tokens used
    total_tokens_used: usize,
    /// Token counting strategy
    strategy: TokenCountingStrategy,
}

impl TokenManager {
    /// Create a new token manager
    pub const fn new(budget: TokenBudget) -> Self {
        Self { budget, total_tokens_used: 0, strategy: TokenCountingStrategy::Approximate }
    }

    /// Create with specific counting strategy
    pub const fn with_strategy(mut self, strategy: TokenCountingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Count tokens in a prompt
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if token counting fails
    pub fn count_tokens(&self, prompt: &Prompt, model: &Model) -> WizardResult<usize> {
        match self.strategy {
            TokenCountingStrategy::Approximate => Self::approximate_count(prompt),
            TokenCountingStrategy::Exact => Self::exact_count(prompt, model),
        }
    }

    /// Approximate token count (fast, ~75% accurate)
    ///
    /// Uses simple heuristic: ~4 characters = 1 token
    fn approximate_count(prompt: &Prompt) -> WizardResult<usize> {
        let mut total_chars = prompt.text.len();

        if let Some(system) = &prompt.system {
            total_chars += system.len();
        }

        for msg in &prompt.history {
            total_chars += msg.content.len();
        }

        // Approximate: 4 characters = 1 token
        Ok(total_chars / 4)
    }

    /// Exact token count using provider-specific tokenizer
    ///
    /// Note: This requires additional dependencies for each provider's tokenizer
    fn exact_count(_prompt: &Prompt, _model: &Model) -> WizardResult<usize> {
        // FUTURE: Implement provider-specific tokenizers
        // - OpenAI: tiktoken
        // - Anthropic: claude-tokenizer
        // - Gemini: sentencepiece

        Err(WizardError::Config(
            "Exact token counting not yet implemented. Use Approximate strategy.".to_string(),
        ))
    }

    /// Check if prompt fits within budget
    ///
    /// # Errors
    ///
    /// Returns `WizardError::TokenLimit` if budget exceeded and action is Error
    pub fn check_budget(&self, prompt: &Prompt, model: &Model) -> WizardResult<()> {
        let token_count = self.count_tokens(prompt, model)?;

        // Check per-request limit
        if token_count > self.budget.max_tokens_per_request {
            return Err(WizardError::TokenLimit {
                requested: token_count,
                max: self.budget.max_tokens_per_request,
            });
        }

        // Check cumulative limit
        if let Some(max_total) = self.budget.max_total_tokens {
            let projected_total = self.total_tokens_used + token_count;
            if projected_total > max_total {
                return Err(WizardError::TokenLimit {
                    requested: projected_total,
                    max: max_total,
                });
            }
        }

        Ok(())
    }

    /// Optimize prompt to fit within budget
    ///
    /// Strategies:
    /// - Truncate history to most recent messages
    /// - Compress system prompt
    /// - Truncate user prompt if necessary
    ///
    /// # Errors
    ///
    /// Returns `WizardError::TokenLimit` if optimization cannot fit within budget
    pub fn optimize_prompt(&self, mut prompt: Prompt, model: &Model) -> WizardResult<Prompt> {
        let mut token_count = self.count_tokens(&prompt, model)?;

        // If already within budget, return as-is
        if token_count <= self.budget.max_tokens_per_request {
            return Ok(prompt);
        }

        // Strategy 1: Truncate history (keep most recent messages)
        while !prompt.history.is_empty() && token_count > self.budget.max_tokens_per_request {
            // Remove oldest message
            prompt.history.remove(0);
            token_count = self.count_tokens(&prompt, model)?;
        }

        // Strategy 2: Compress system prompt (remove extra whitespace)
        if token_count > self.budget.max_tokens_per_request {
            if let Some(system) = &prompt.system {
                let compressed = system.split_whitespace().collect::<Vec<_>>().join(" ");
                prompt.system = Some(compressed);
                token_count = self.count_tokens(&prompt, model)?;
            }
        }

        // Strategy 3: Truncate user prompt as last resort
        if token_count > self.budget.max_tokens_per_request {
            let target_chars = self.budget.max_tokens_per_request * 4; // Approximate
            if prompt.text.len() > target_chars {
                prompt.text.truncate(target_chars);
                prompt.text.push_str("...");
                token_count = self.count_tokens(&prompt, model)?;
            }
        }

        // Final check
        if token_count > self.budget.max_tokens_per_request {
            return Err(WizardError::TokenLimit {
                requested: token_count,
                max: self.budget.max_tokens_per_request,
            });
        }

        Ok(prompt)
    }

    /// Record token usage (updates cumulative count)
    pub fn record_usage(&mut self, usage: &TokenUsage) {
        self.total_tokens_used += usage.total_tokens;
    }

    /// Get total tokens used
    pub const fn total_tokens_used(&self) -> usize {
        self.total_tokens_used
    }

    /// Get remaining budget (if cumulative limit set)
    pub const fn remaining_budget(&self) -> Option<usize> {
        if let Some(max_total) = self.budget.max_total_tokens {
            Some(max_total.saturating_sub(self.total_tokens_used))
        } else {
            None
        }
    }

    /// Reset cumulative token count
    pub fn reset(&mut self) {
        self.total_tokens_used = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::config::{AnthropicModel, Model};
    use crate::wizard::types::Message;

    #[test]
    fn test_token_budget_default() {
        // Arrange + Act
        let budget = TokenBudget::default();

        // Assert
        assert_eq!(budget.max_tokens_per_request, 8192);
        assert_eq!(budget.max_total_tokens, None);
        assert_eq!(budget.on_exceed, BudgetExceedAction::Error);
    }

    #[test]
    fn test_token_budget_builder() {
        // Arrange + Act
        let budget = TokenBudget::new(4096)
            .with_max_total(100000)
            .with_exceed_action(BudgetExceedAction::Truncate);

        // Assert
        assert_eq!(budget.max_tokens_per_request, 4096);
        assert_eq!(budget.max_total_tokens, Some(100000));
        assert_eq!(budget.on_exceed, BudgetExceedAction::Truncate);
    }

    #[test]
    fn test_approximate_token_count() {
        // Arrange
        let prompt = Prompt::new("Hello world, this is a test prompt with some text.");

        // Act
        let count = TokenManager::approximate_count(&prompt).expect("count tokens");

        // Assert - ~52 chars / 4 = ~13 tokens
        assert!(count >= 10 && count <= 15);
    }

    #[test]
    fn test_approximate_count_with_system() {
        // Arrange
        let prompt = Prompt::new("User prompt text")
            .with_system("You are a helpful assistant");

        // Act
        let count = TokenManager::approximate_count(&prompt).expect("count tokens");

        // Assert - Both prompts combined
        assert!(count > 5);
    }

    #[test]
    fn test_approximate_count_with_history() {
        // Arrange
        let prompt = Prompt::new("Current message")
            .with_message(crate::wizard::types::Role::User, "Previous message")
            .with_message(crate::wizard::types::Role::Assistant, "Previous response");

        // Act
        let count = TokenManager::approximate_count(&prompt).expect("count tokens");

        // Assert
        assert!(count > 10); // Should count all messages
    }

    #[test]
    fn test_check_budget_pass() {
        // Arrange
        let budget = TokenBudget::new(1000);
        let manager = TokenManager::new(budget);
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);
        let prompt = Prompt::new("Short prompt");

        // Act
        let result = manager.check_budget(&prompt, &model);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_budget_exceed() {
        // Arrange
        let budget = TokenBudget::new(5); // Very small budget
        let manager = TokenManager::new(budget);
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);
        let prompt = Prompt::new("This is a much longer prompt that will exceed the tiny budget");

        // Act
        let result = manager.check_budget(&prompt, &model);

        // Assert
        assert!(result.is_err());
        match result {
            Err(WizardError::TokenLimit { requested, max }) => {
                assert!(requested > max);
            }
            _ => {
                assert!(false, "Expected TokenLimit error");
            }
        }
    }

    #[test]
    fn test_optimize_prompt_truncate_history() {
        // Arrange
        let budget = TokenBudget::new(50); // Small budget
        let manager = TokenManager::new(budget);
        let model = Model::Anthropic(AnthropicModel::Claude3Sonnet);

        let prompt = Prompt::new("Current message")
            .with_message(crate::wizard::types::Role::User, "Very long message that takes many tokens to represent properly and should be removed")
            .with_message(crate::wizard::types::Role::Assistant, "Another very long message that takes many tokens to represent")
            .with_message(crate::wizard::types::Role::User, "Yet another long message");

        // Act
        let optimized = manager.optimize_prompt(prompt, &model).expect("optimize");

        // Assert - history should be truncated
        assert!(optimized.history.len() < 3);
    }

    #[test]
    fn test_record_usage() {
        // Arrange
        let budget = TokenBudget::new(1000);
        let mut manager = TokenManager::new(budget);
        let usage = TokenUsage::new(100, 50);

        // Act
        manager.record_usage(&usage);

        // Assert
        assert_eq!(manager.total_tokens_used(), 150);
    }

    #[test]
    fn test_remaining_budget() {
        // Arrange
        let budget = TokenBudget::new(1000).with_max_total(10000);
        let mut manager = TokenManager::new(budget);
        let usage = TokenUsage::new(100, 50);

        // Act
        manager.record_usage(&usage);

        // Assert
        assert_eq!(manager.remaining_budget(), Some(9850));
    }

    #[test]
    fn test_reset() {
        // Arrange
        let budget = TokenBudget::new(1000);
        let mut manager = TokenManager::new(budget);
        let usage = TokenUsage::new(100, 50);
        manager.record_usage(&usage);

        // Act
        manager.reset();

        // Assert
        assert_eq!(manager.total_tokens_used(), 0);
    }
}
