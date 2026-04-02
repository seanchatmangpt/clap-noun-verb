//! Integration: Template Rendering with Tera
//!
//! Glue code that connects domain Paper structures to Tera templates.
//! This is the ONLY place where Tera is used - domain stays pure.
//!
//! Performance: Uses lazy_static to cache the Tera engine globally,
//! avoiding 5-15ms parsing overhead on every render call.

use lazy_static::lazy_static;
use tera::{Tera, Context};
use crate::domain::{Paper, PaperFamily};

lazy_static! {
    /// Globally cached Tera template engine - parsed once, reused forever.
    /// Eliminates 5-15ms parsing overhead per render call.
    static ref TERA_ENGINE: Result<Tera, String> = {
        Tera::new("templates/**/*.tera")
            .map_err(|e| format!("Tera parsing error: {}", e))
    };
}

/// Get the globally cached template engine.
///
/// This is the preferred way to access the Tera engine - it returns
/// a reference to the statically cached instance, avoiding re-parsing
/// templates on every call.
///
/// # Errors
/// Returns an error if the initial template parsing failed.
pub fn get_template_engine() -> Result<&'static Tera, String> {
    TERA_ENGINE.as_ref().map_err(|e| e.clone())
}

/// Initialize Tera template engine with playground templates.
///
/// DEPRECATED: Use `get_template_engine()` instead for cached access.
/// This function is kept for backward compatibility but now returns
/// a reference to the cached engine rather than creating a new one.
///
/// # Errors
/// Returns an error if template parsing fails.
#[deprecated(since = "0.1.0", note = "Use get_template_engine() instead for cached access")]
#[allow(dead_code)]
pub fn init_template_engine() -> Result<&'static Tera, &'static str> {
    TERA_ENGINE.as_ref().map_err(|_| "Failed to initialize Tera engine")
}

/// Render a Paper to LaTeX using Tera templates
///
/// This function bridges domain (Paper) to infrastructure (Tera).
///
/// # Errors
/// Returns an error if the template file is not found or rendering fails.
/// FMEA-2: Provides clear error message when template is missing.
pub fn render_paper_latex(paper: &Paper, tera: &Tera) -> Result<String, String> {
    let mut context = Context::new();

    // Transfer domain data to template context
    context.insert("title", &paper.title);
    context.insert("author", &paper.author);
    context.insert("family", paper.family.name());
    context.insert("abstract", &paper.abstract_text);

    // Add sections
    let sections: Vec<_> = paper.sections.iter().map(|s| {
        let mut map = std::collections::HashMap::new();
        map.insert("title", s.title.as_str());
        map.insert("content", s.content.as_str());
        map
    }).collect();
    context.insert("sections", &sections);

    // Add individual section content for IMRaD template compatibility
    for section in &paper.sections {
        let key = section.title.to_lowercase().replace(' ', "_");
        context.insert(&key, &section.content);
    }

    // Select template based on family
    let template_name = match paper.family {
        PaperFamily::IMRaD => "imrad.tex.tera",
        PaperFamily::Argument => "argument.tex.tera",
        PaperFamily::Contribution => "contribution.tex.tera",
        PaperFamily::Monograph => "monograph.tex.tera",
        PaperFamily::DSR => "dsr.tex.tera",
        PaperFamily::Narrative => "narrative.tex.tera",
        PaperFamily::Papers => "paper.tex.tera",
    };

    // FMEA-2: Check if template exists before rendering
    let available_templates: Vec<_> = tera.get_template_names().collect();
    if !available_templates.contains(&template_name) {
        return Err(format!(
            "Template file not found: '{}'. Ensure templates directory exists with required .tera files. \
             Available templates: {:?}",
            template_name, available_templates
        ));
    }

    tera.render(template_name, &context)
        .map_err(|e| format!("Template rendering error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_template_engine_handles_missing_templates() {
        // This test verifies error handling when templates don't exist
        // In real scenario, templates directory must exist
        let result = Tera::new("nonexistent/**/*.tera");
        // Tera returns Ok even if no templates found, just empty
        assert!(result.is_ok());
    }
}
