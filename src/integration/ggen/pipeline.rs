//! Template processing pipeline integration

use crate::integration::ggen::error::{GgenError, GgenResult};

/// Wrapper around ggen-core Pipeline for template processing
///
/// Provides an ergonomic interface for template rendering pipelines.
pub struct GgenPipeline {
    inner: ggen_core::Pipeline,
}

impl GgenPipeline {
    /// Create a new template processing pipeline
    pub fn new() -> GgenResult<Self> {
        let pipeline = ggen_core::Pipeline::new().map_err(|e| GgenError::Core(e))?;
        Ok(Self { inner: pipeline })
    }

    /// Get the inner ggen-core Pipeline for advanced usage
    pub fn inner(&self) -> &ggen_core::Pipeline {
        &self.inner
    }

    /// Get a mutable reference to the inner pipeline
    pub fn inner_mut(&mut self) -> &mut ggen_core::Pipeline {
        &mut self.inner
    }

    /// Consume self and return the inner pipeline
    pub fn into_inner(self) -> ggen_core::Pipeline {
        self.inner
    }
}

impl Default for GgenPipeline {
    fn default() -> Self {
        Self::new().expect("Failed to create default pipeline")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        // Arrange & Act
        let pipeline = GgenPipeline::new();

        // Assert
        assert!(pipeline.is_ok());
    }

    #[test]
    fn test_pipeline_default() {
        // Arrange & Act
        let pipeline = GgenPipeline::default();

        // Assert - should not panic
        let _ = pipeline.inner();
    }

    #[test]
    fn test_pipeline_into_inner() {
        // Arrange
        let pipeline = GgenPipeline::new().unwrap();

        // Act
        let inner = pipeline.into_inner();

        // Assert - should be able to use the inner pipeline
        let _ = inner;
    }
}
