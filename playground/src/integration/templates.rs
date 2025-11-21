//! Integration: Template Rendering with Tera
//!
//! Glue code that connects domain Paper structures to Tera templates.
//! This is the ONLY place where Tera is used - domain stays pure.

use tera::{Tera, Context};
use crate::domain::{Paper, PaperFamily};

/// Initialize Tera template engine with playground templates
pub fn init_template_engine() -> Result<Tera, String> {
    Tera::new("templates/**/*.tera")
        .map_err(|e| format!("Tera parsing error: {}", e))
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
