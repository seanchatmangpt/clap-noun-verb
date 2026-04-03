//! Integration: Template Rendering with Tera
//!
//! Glue code that connects domain Paper structures and ggen pack templates to Tera.
//! This is the ONLY place where Tera is used - domain stays pure.
//!
//! Performance: Uses lazy_static to cache the Tera engine globally,
//! avoiding 5-15ms parsing overhead on every render call.
//!
//! ggen pack templates use one-shot Tera instances since they come from
//! installed packs rather than the global templates/ directory.

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

/// Render a template string with variables using a one-shot Tera instance.
///
/// Used for ggen pack template rendering where templates come from installed packs
/// rather than the global templates directory. Each call creates a fresh Tera
/// instance, parses the template content, and renders it with the provided variables.
///
/// # Arguments
/// * `template_content` - The raw Tera template string (e.g. from a pack's template file)
/// * `variables` - Key-value pairs to inject into the template context
///
/// # Errors
/// Returns an error if the template content fails to parse or rendering fails.
pub fn render_template_string(
    template_content: &str,
    variables: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let mut tera = Tera::default();
    tera.add_raw_template("inline", template_content)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    let mut context = Context::new();
    for (key, value) in variables {
        context.insert(key, value);
    }

    tera.render("inline", &context)
        .map_err(|e| format!("Template rendering error: {}", e))
}

/// Render a named ggen template from the templates/ directory using the cached engine.
///
/// This uses the globally cached `TERA_ENGINE` to render templates that live in
/// the `templates/` directory (e.g. pack scaffolding templates, code generation
/// templates registered at build time).
///
/// # Arguments
/// * `template_name` - Name of the template file (e.g. `"pack_config.tera"`)
/// * `variables` - Key-value pairs to inject into the template context
///
/// # Errors
/// Returns an error if the template engine failed to initialize, the named
/// template does not exist, or rendering fails.
pub fn render_ggen_template(
    template_name: &str,
    variables: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let tera = get_template_engine()?;

    let mut context = Context::new();
    for (key, value) in variables {
        context.insert(key, value);
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

    #[test]
    fn test_render_template_string_basic() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("name".to_string(), "pack_one".to_string());
        vars.insert("version".to_string(), "1.0.0".to_string());

        let result = render_template_string(
            "// {{ name }} v{{ version }}",
            &vars,
        );
        assert_eq!(result.unwrap(), "// pack_one v1.0.0");
    }

    #[test]
    fn test_render_template_string_with_conditionals() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("has_auth".to_string(), "true".to_string());
        vars.insert("module".to_string(), "auth".to_string());

        let result = render_template_string(
            "{% if has_auth == \"true\" %}mod {{ module }};{% endif %}",
            &vars,
        );
        assert_eq!(result.unwrap(), "mod auth;");
    }

    #[test]
    fn test_render_template_string_invalid_syntax() {
        let vars = std::collections::HashMap::new();
        let result = render_template_string("{% invalid block %}", &vars);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse template"));
    }

    #[test]
    fn test_render_template_string_empty() {
        let vars = std::collections::HashMap::new();
        let result = render_template_string("hello world", &vars);
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_render_ggen_template_missing() {
        let vars = std::collections::HashMap::new();
        let result = render_ggen_template("nonexistent_template.tera", &vars);
        assert!(result.is_err());
    }
}
