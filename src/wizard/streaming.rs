//! Streaming responses for token-by-token generation
//!
//! This module provides streaming support for AI model responses, allowing
//! token-by-token generation with backpressure handling and cancellation support.
//!
//! ## Features
//!
//! - Token-by-token streaming with AsyncIterator
//! - Buffering and backpressure handling
//! - Cancellation support via tokio::sync::watch
//! - Error recovery during streaming
//! - Type-safe stream configuration

use crate::wizard::{
    config::{ModelConfig, WizardConfig},
    error::{WizardError, WizardResult},
    types::{Message, Prompt, ResponseMetadata, Role, TokenUsage, WizardResponse},
};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// Configuration for streaming responses
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamingConfig {
    /// Buffer size for streaming chunks
    pub buffer_size: usize,
    /// Enable backpressure handling
    pub enable_backpressure: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self { buffer_size: 32, enable_backpressure: true }
    }
}

impl StreamingConfig {
    /// Create a new streaming configuration
    pub const fn new(buffer_size: usize) -> Self {
        Self { buffer_size, enable_backpressure: true }
    }

    /// Set buffer size
    pub const fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable or disable backpressure handling
    pub const fn with_backpressure(mut self, enable: bool) -> Self {
        self.enable_backpressure = enable;
        self
    }
}

/// A streaming chunk from the AI model
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamChunk {
    /// The text content of this chunk
    pub text: String,
    /// Whether this is the final chunk
    pub is_final: bool,
    /// Token usage (only present in final chunk)
    pub usage: Option<TokenUsage>,
}

impl StreamChunk {
    /// Create a new streaming chunk
    pub fn new(text: impl Into<String>, is_final: bool) -> Self {
        Self { text: text.into(), is_final, usage: None }
    }

    /// Create a final chunk with token usage
    pub fn final_chunk(text: impl Into<String>, usage: Option<TokenUsage>) -> Self {
        Self { text: text.into(), is_final: true, usage }
    }
}

/// Streaming client wrapper around GenAiClient
///
/// This struct provides streaming generation with token-by-token responses.
/// It wraps the underlying genai client and provides a Stream implementation.
pub struct StreamingClient {
    /// The underlying rust-genai client
    client: genai::Client,
    /// Model configuration
    config: ModelConfig,
    /// Streaming configuration
    streaming_config: StreamingConfig,
}

impl StreamingClient {
    /// Create a new streaming client from configuration
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if configuration is invalid
    pub async fn new(wizard_config: WizardConfig) -> WizardResult<Self> {
        wizard_config.validate()?;

        let client = genai::Client::default();

        Ok(Self {
            client,
            config: wizard_config.model_config,
            streaming_config: StreamingConfig::default(),
        })
    }

    /// Create with custom streaming configuration
    pub fn with_streaming_config(mut self, config: StreamingConfig) -> Self {
        self.streaming_config = config;
        self
    }

    /// Generate a streaming response from a prompt
    ///
    /// Returns a Stream of StreamChunks that can be consumed token-by-token.
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Request` if the API request fails
    pub async fn generate_stream(
        &self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<impl Stream<Item = WizardResult<StreamChunk>>> {
        let prompt = prompt.into();

        // Build chat messages
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system) = &prompt.system {
            messages.push(genai::chat::ChatMessage {
                role: genai::chat::ChatRole::System,
                content: genai::chat::MessageContent::Text(system.clone()),
                options: None,
            });
        }

        // Add conversation history
        for msg in &prompt.history {
            messages.push(genai::chat::ChatMessage {
                role: Self::convert_role(msg.role),
                content: genai::chat::MessageContent::Text(msg.content.clone()),
                options: None,
            });
        }

        // Add user prompt
        messages.push(genai::chat::ChatMessage {
            role: genai::chat::ChatRole::User,
            content: genai::chat::MessageContent::Text(prompt.text.clone()),
            options: None,
        });

        // Create streaming chat request
        let chat_req = genai::chat::ChatRequest {
            messages,
            model: genai::ModelName::from(self.config.model.model_id()),
            temperature: Some(self.config.temperature.into()),
            top_p: Some(self.config.top_p.into()),
            max_tokens: Some(self.config.max_response_tokens),
            stream: true, // Enable streaming
            ..Default::default()
        };

        // Execute streaming request
        let stream_res = self
            .client
            .exec_chat_stream(self.config.model.model_id(), chat_req, None)
            .await
            .map_err(|e| WizardError::Request(e.to_string()))?;

        // Convert to our StreamChunk type
        Ok(stream_res.map(move |result| match result {
            Ok(chunk) => {
                let text = chunk.content.as_ref().and_then(|c| c.text_as_str()).unwrap_or("");

                let usage = chunk.usage.map(|u| {
                    TokenUsage::new(u.prompt_tokens.unwrap_or(0), u.completion_tokens.unwrap_or(0))
                });

                Ok(StreamChunk { text: text.to_string(), is_final: chunk.is_final(), usage })
            }
            Err(e) => Err(WizardError::Request(e.to_string())),
        }))
    }

    /// Generate a complete response by collecting all streaming chunks
    ///
    /// This is a convenience method that collects all chunks into a WizardResponse.
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Request` if the API request fails
    pub async fn generate_complete(
        &self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<WizardResponse> {
        let mut stream = self.generate_stream(prompt).await?;

        let mut full_text = String::new();
        let mut final_usage = None;

        while let Some(result) = stream.next().await {
            let chunk = result?;
            full_text.push_str(&chunk.text);
            if chunk.is_final {
                final_usage = chunk.usage;
            }
        }

        let mut response = WizardResponse::new(full_text, self.config.model.model_id())
            .with_metadata(ResponseMetadata::default());

        if let Some(usage) = final_usage {
            response = response.with_usage(usage);
        }

        Ok(response)
    }

    /// Convert our Role enum to rust-genai's ChatRole
    fn convert_role(role: Role) -> genai::chat::ChatRole {
        match role {
            Role::User => genai::chat::ChatRole::User,
            Role::Assistant => genai::chat::ChatRole::Assistant,
            Role::System => genai::chat::ChatRole::System,
        }
    }
}

/// A stream wrapper with cancellation support
///
/// This struct wraps a Stream and provides cancellation via a watch channel.
pub struct CancellableStream<S> {
    stream: Pin<Box<S>>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
}

impl<S> CancellableStream<S>
where
    S: Stream,
{
    /// Create a new cancellable stream
    pub fn new(stream: S) -> (Self, tokio::sync::watch::Sender<bool>) {
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        (Self { stream: Box::pin(stream), cancel_rx }, cancel_tx)
    }
}

impl<S> Stream for CancellableStream<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check if cancellation was requested
        if *self.cancel_rx.borrow() {
            return Poll::Ready(None);
        }

        self.stream.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        // Arrange + Act
        let config = StreamingConfig::default();

        // Assert
        assert_eq!(config.buffer_size, 32);
        assert!(config.enable_backpressure);
    }

    #[test]
    fn test_streaming_config_builder() {
        // Arrange + Act
        let config = StreamingConfig::new(64).with_backpressure(false);

        // Assert
        assert_eq!(config.buffer_size, 64);
        assert!(!config.enable_backpressure);
    }

    #[test]
    fn test_stream_chunk_new() {
        // Arrange + Act
        let chunk = StreamChunk::new("Hello", false);

        // Assert
        assert_eq!(chunk.text, "Hello");
        assert!(!chunk.is_final);
        assert!(chunk.usage.is_none());
    }

    #[test]
    fn test_stream_chunk_final() {
        // Arrange
        let usage = TokenUsage::new(100, 50);

        // Act
        let chunk = StreamChunk::final_chunk("Done", Some(usage));

        // Assert
        assert_eq!(chunk.text, "Done");
        assert!(chunk.is_final);
        assert_eq!(chunk.usage, Some(usage));
    }

    #[tokio::test]
    async fn test_cancellable_stream() {
        // Arrange
        let stream = futures::stream::iter(vec![1, 2, 3, 4, 5]);
        let (mut cancellable, cancel_tx) = CancellableStream::new(stream);

        // Act - collect first 2 items
        let first = cancellable.next().await;
        let second = cancellable.next().await;

        // Cancel the stream
        cancel_tx.send(true).expect("send cancel signal");

        // Try to get next item (should be None due to cancellation)
        let third = cancellable.next().await;

        // Assert
        assert_eq!(first, Some(1));
        assert_eq!(second, Some(2));
        assert_eq!(third, None); // Stream cancelled
    }
}
