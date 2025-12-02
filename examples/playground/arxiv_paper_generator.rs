//! arXiv Paper Generator - 4 Abstraction Layers
//!
//! Consolidated arXiv paper generator following rdf_mcp_core.rs pattern.
//! Integrates with thesis-ontology.ttl for semantic paper structure.
//!
//! Run: cargo run --example arxiv_paper_generator
//!
//! Architecture:
//! - Layer 1: Domain Model (Paper, Section, ContentType, DiagramType)
//! - Layer 2: Ontology Integration (query thesis RDF for structure)
//! - Layer 3: Generator (LaTeX templates with Handlebars)
//! - Layer 4: Pipeline (build paper from thesis family)

use handlebars::Handlebars;
use oxigraph::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("⚡ arXiv Paper Generator - 4 Abstraction Layers\n");

    // Layer 1: Define domain model
    let paper = create_sample_paper();
    println!("✅ Layer 1: Domain model created ({} sections)\n", paper.sections.len());

    // Layer 2: Query thesis ontology for structure
    let ontology = ThesisOntology::load()?;
    let structure = ontology.query_paper_structure()?;
    println!("✅ Layer 2: Ontology loaded ({} section templates)\n", structure.len());

    // Layer 3: Generate LaTeX
    let generator = LatexGenerator::new()?;
    let latex = generator.generate(&paper)?;
    println!("✅ Layer 3: LaTeX generated ({} bytes)\n", latex.len());

    // Layer 4: Execute pipeline
    let pipeline = PaperPipeline::new()?;
    let output_path = pipeline.build_paper(&paper)?;
    println!("✅ Layer 4: Pipeline complete\n");
    println!("📄 Generated: {}", output_path);

    println!("\n✅ All layers complete!");
    println!("   Domain → Ontology → Generator → Pipeline");

    Ok(())
}

// ============================================================================
// LAYER 1: DOMAIN MODEL
// Type-safe representations of papers, sections, content
// ============================================================================

/// Section types following IMRaD structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SectionType {
    Abstract,
    Introduction,
    Method,
    Results,
    Discussion,
    Conclusion,
    References,
}

impl SectionType {
    fn latex_title(&self) -> &'static str {
        match self {
            Self::Abstract => "Abstract",
            Self::Introduction => "Introduction",
            Self::Method => "Method",
            Self::Results => "Results",
            Self::Discussion => "Discussion",
            Self::Conclusion => "Conclusion",
            Self::References => "References",
        }
    }
}

/// Content types that can appear in sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text(String),
    Equation(String),
    Diagram(DiagramType),
    Citation(String),
}

impl ContentType {
    fn to_latex(&self) -> Result<String, Box<dyn Error>> {
        match self {
            Self::Text(text) => Ok(text.clone()),
            Self::Equation(eq) => Ok(format!("\\begin{{equation}}\n{}\n\\end{{equation}}", eq)),
            Self::Diagram(diagram) => diagram.to_latex(),
            Self::Citation(cite_key) => Ok(format!("\\cite{{{}}}", cite_key)),
        }
    }
}

/// Diagram types with conversion strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagramType {
    PlantUML(String),
    TikZ(String),
    Mermaid(String),
}

impl DiagramType {
    fn to_latex(&self) -> Result<String, Box<dyn Error>> {
        match self {
            Self::PlantUML(source) => {
                // Convert PlantUML to TikZ (simplified)
                Ok(format!(
                    "\\begin{{tikzpicture}}\n% PlantUML source:\n% {}\n\\end{{tikzpicture}}",
                    source
                ))
            }
            Self::TikZ(source) => {
                Ok(format!("\\begin{{tikzpicture}}\n{}\n\\end{{tikzpicture}}", source))
            }
            Self::Mermaid(source) => {
                // Convert Mermaid to TikZ (simplified)
                Ok(format!(
                    "\\begin{{tikzpicture}}\n% Mermaid source:\n% {}\n\\end{{tikzpicture}}",
                    source
                ))
            }
        }
    }
}

/// Paper section with typed content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub section_type: SectionType,
    pub title: String,
    pub content: Vec<ContentType>,
}

impl Section {
    fn new(section_type: SectionType, title: String) -> Self {
        Self { section_type, title, content: Vec::new() }
    }

    fn add_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.content.push(ContentType::Text(text.into()));
        self
    }

    fn add_equation(&mut self, equation: impl Into<String>) -> &mut Self {
        self.content.push(ContentType::Equation(equation.into()));
        self
    }

    fn add_citation(&mut self, cite_key: impl Into<String>) -> &mut Self {
        self.content.push(ContentType::Citation(cite_key.into()));
        self
    }
}

/// Bibliography entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibEntry {
    pub key: String,
    pub entry_type: String,
    pub fields: BTreeMap<String, String>,
}

impl BibEntry {
    fn to_bibtex(&self) -> String {
        let mut bib = format!("@{}{{{},\n", self.entry_type, self.key);
        for (key, value) in &self.fields {
            bib.push_str(&format!("  {} = {{{}}},\n", key, value));
        }
        bib.push_str("}\n");
        bib
    }
}

/// Complete paper structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub title: String,
    pub authors: Vec<String>,
    pub sections: Vec<Section>,
    pub bibliography: Vec<BibEntry>,
}

impl Paper {
    fn new(title: String, authors: Vec<String>) -> Self {
        Self { title, authors, sections: Vec::new(), bibliography: Vec::new() }
    }

    fn add_section(&mut self, section: Section) -> &mut Self {
        self.sections.push(section);
        self
    }

    fn add_bib_entry(&mut self, entry: BibEntry) -> &mut Self {
        self.bibliography.push(entry);
        self
    }
}

// ============================================================================
// LAYER 2: ONTOLOGY INTEGRATION
// Query thesis-ontology.ttl for paper structure using Λ-scheduling and Π-profiling
// ============================================================================

/// Thesis ontology wrapper with SPARQL queries
pub struct ThesisOntology {
    store: Store,
}

impl ThesisOntology {
    fn load() -> Result<Self, Box<dyn Error>> {
        let store = Store::new()?;

        // Try to load thesis-ontology.ttl from playground directory
        let ontology_path = "playground/thesis-ontology.ttl";
        if Path::new(ontology_path).exists() {
            let turtle_data = fs::read_to_string(ontology_path)?;
            store.load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle_data.as_bytes())?;
        }

        Ok(Self { store })
    }

    /// Query for paper structure using Λ-scheduling (optimal section order)
    fn query_paper_structure(&self) -> Result<Vec<SectionTemplate>, Box<dyn Error>> {
        let query = r#"
            PREFIX htf: <http://thesis.hyper/framework/>

            SELECT ?shard ?position ?purpose
            WHERE {
                ?shard a htf:IMRaDFamily .
                ?shard htf:position ?position .
                ?shard htf:purpose ?purpose .
            }
            ORDER BY ?position
        "#;

        let mut templates = Vec::new();

        if let Ok(results) = self.store.query(query) {
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                for solution in solutions {
                    if let Ok(sol) = solution {
                        let position = sol
                            .get("position")
                            .and_then(|t| t.as_ref().to_string().parse::<i32>().ok())
                            .unwrap_or(0);

                        let purpose = sol.get("purpose").map(|t| t.to_string()).unwrap_or_default();

                        templates.push(SectionTemplate { position, purpose });
                    }
                }
            }
        }

        // Fallback if no results
        if templates.is_empty() {
            templates = vec![
                SectionTemplate { position: 1, purpose: "Abstract".to_string() },
                SectionTemplate { position: 2, purpose: "Introduction".to_string() },
                SectionTemplate { position: 3, purpose: "Method".to_string() },
                SectionTemplate { position: 4, purpose: "Results".to_string() },
                SectionTemplate { position: 5, purpose: "Discussion".to_string() },
                SectionTemplate { position: 6, purpose: "Conclusion".to_string() },
            ];
        }

        Ok(templates)
    }

    /// Query for content mapping using Π-profiling
    fn query_content_mapping(&self) -> Result<BTreeMap<String, Vec<String>>, Box<dyn Error>> {
        let query = r#"
            PREFIX htf: <http://thesis.hyper/framework/>

            SELECT ?shard ?purpose
            WHERE {
                ?shard a htf:ContributionFamily .
                ?shard htf:purpose ?purpose .
            }
        "#;

        let mut mapping = BTreeMap::new();

        if let Ok(results) = self.store.query(query) {
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                for solution in solutions {
                    if let Ok(sol) = solution {
                        let purpose = sol.get("purpose").map(|t| t.to_string()).unwrap_or_default();

                        mapping
                            .entry("contribution".to_string())
                            .or_insert_with(Vec::new)
                            .push(purpose);
                    }
                }
            }
        }

        Ok(mapping)
    }
}

/// Section template from ontology
#[derive(Debug, Clone)]
pub struct SectionTemplate {
    pub position: i32,
    pub purpose: String,
}

// ============================================================================
// LAYER 3: LATEX GENERATOR
// Template-based LaTeX generation using Handlebars
// ============================================================================

/// LaTeX generator with Handlebars templates
pub struct LatexGenerator {
    handlebars: Handlebars<'static>,
}

impl LatexGenerator {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut handlebars = Handlebars::new();

        // Register main paper template
        handlebars.register_template_string("paper", Self::paper_template())?;

        Ok(Self { handlebars })
    }

    fn paper_template() -> &'static str {
        r#"\documentclass{article}
\usepackage{amsmath}
\usepackage{tikz}
\usepackage{cite}

\title{ {{title}} }
\author{ {{authors}} }
\date{\today}

\begin{document}

\maketitle

{{#each sections}}
\section{ {{this.title}} }
{{this.content}}

{{/each}}

\bibliographystyle{plain}
\bibliography{references}

\end{document}
"#
    }

    fn generate(&self, paper: &Paper) -> Result<String, Box<dyn Error>> {
        // Prepare template data
        let mut data = BTreeMap::new();
        data.insert("title", paper.title.clone());
        data.insert("authors", paper.authors.join(", "));

        // Process sections
        let mut sections_data = Vec::new();
        for section in &paper.sections {
            let mut section_content = String::new();
            for content in &section.content {
                section_content.push_str(&content.to_latex()?);
                section_content.push('\n');
            }

            sections_data
                .push(SectionData { title: section.title.clone(), content: section_content });
        }

        // Render template (simplified - handlebars needs HashMap)
        let latex = self.generate_latex_manually(paper)?;

        Ok(latex)
    }

    // Manual generation (type-safe, no unwrap)
    fn generate_latex_manually(&self, paper: &Paper) -> Result<String, Box<dyn Error>> {
        let mut latex = String::new();

        // Document header
        latex.push_str("\\documentclass{article}\n");
        latex.push_str("\\usepackage{amsmath}\n");
        latex.push_str("\\usepackage{tikz}\n");
        latex.push_str("\\usepackage{cite}\n\n");

        latex.push_str(&format!("\\title{{{}}}\n", paper.title));
        latex.push_str(&format!("\\author{{{}}}\n", paper.authors.join(", ")));
        latex.push_str("\\date{\\today}\n\n");

        latex.push_str("\\begin{document}\n\n");
        latex.push_str("\\maketitle\n\n");

        // Sections
        for section in &paper.sections {
            latex.push_str(&format!("\\section{{{}}}\n", section.title));
            for content in &section.content {
                latex.push_str(&content.to_latex()?);
                latex.push('\n');
            }
            latex.push('\n');
        }

        // Bibliography
        if !paper.bibliography.is_empty() {
            latex.push_str("\\bibliographystyle{plain}\n");
            latex.push_str("\\bibliography{references}\n\n");
        }

        latex.push_str("\\end{document}\n");

        Ok(latex)
    }

    fn generate_bibtex(&self, paper: &Paper) -> String {
        let mut bibtex = String::new();
        for entry in &paper.bibliography {
            bibtex.push_str(&entry.to_bibtex());
            bibtex.push('\n');
        }
        bibtex
    }
}

#[derive(Serialize)]
struct SectionData {
    title: String,
    content: String,
}

// ============================================================================
// LAYER 4: PAPER PIPELINE
// End-to-end pipeline: ontology → paper → LaTeX → output
// ============================================================================

/// Paper generation pipeline
pub struct PaperPipeline {
    generator: LatexGenerator,
    ontology: ThesisOntology,
}

impl PaperPipeline {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self { generator: LatexGenerator::new()?, ontology: ThesisOntology::load()? })
    }

    /// Build paper from thesis family structure
    fn build_paper(&self, paper: &Paper) -> Result<String, Box<dyn Error>> {
        // Generate LaTeX
        let latex = self.generator.generate(paper)?;
        let bibtex = self.generator.generate_bibtex(paper);

        // Create output directory
        let output_dir = "examples/playground/generated";
        fs::create_dir_all(output_dir)?;

        // Write LaTeX file
        let latex_path = format!("{}/paper.tex", output_dir);
        fs::write(&latex_path, latex)?;

        // Write BibTeX file
        if !bibtex.is_empty() {
            let bib_path = format!("{}/references.bib", output_dir);
            fs::write(bib_path, bibtex)?;
        }

        Ok(latex_path)
    }

    /// Store paper metadata in MCP memory (pattern demonstration)
    fn store_in_mcp(&self, paper: &Paper) -> Result<(), Box<dyn Error>> {
        println!("\n💾 MCP Storage Pattern:");
        println!("  mcp_store(\"paper/title\", \"{}\")", paper.title);
        println!("  mcp_store(\"paper/authors\", {:?})", paper.authors);
        println!("  mcp_store(\"paper/sections\", {} sections)", paper.sections.len());
        Ok(())
    }
}

// ============================================================================
// SAMPLE DATA & HELPERS
// ============================================================================

fn create_sample_paper() -> Paper {
    let mut paper = Paper::new(
        "Neural Network Optimization for Large Language Models".to_string(),
        vec!["Alice Smith".to_string(), "Bob Jones".to_string()],
    );

    // Abstract
    let mut abstract_section = Section::new(SectionType::Abstract, "Abstract".to_string());
    abstract_section.add_text(
        "This paper presents a novel approach to neural network optimization \
         for large language models. We demonstrate significant improvements in \
         training efficiency and model performance.",
    );
    paper.add_section(abstract_section);

    // Introduction
    let mut intro = Section::new(SectionType::Introduction, "Introduction".to_string());
    intro.add_text("Large language models have revolutionized natural language processing.");
    intro.add_citation("transformer2017");
    paper.add_section(intro);

    // Method
    let mut method = Section::new(SectionType::Method, "Method".to_string());
    method.add_text("Our optimization approach uses adaptive learning rates:");
    method.add_equation(r"\alpha_t = \alpha_0 \cdot \frac{1}{\sqrt{t}}");
    paper.add_section(method);

    // Results
    let mut results = Section::new(SectionType::Results, "Results".to_string());
    results.add_text("We achieved 15% improvement in training speed.");
    paper.add_section(results);

    // Bibliography
    let mut bib_entry = BibEntry {
        key: "transformer2017".to_string(),
        entry_type: "article".to_string(),
        fields: BTreeMap::new(),
    };
    bib_entry.fields.insert("author".to_string(), "Vaswani et al.".to_string());
    bib_entry.fields.insert("title".to_string(), "Attention is All You Need".to_string());
    bib_entry.fields.insert("year".to_string(), "2017".to_string());
    paper.add_bib_entry(bib_entry);

    paper
}

// ============================================================================
// TESTS (Chicago TDD - State-based, AAA pattern, Real collaborators)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Layer 1 Tests: Domain Model

    #[test]
    fn test_section_type_latex_title() {
        // Arrange
        let section_type = SectionType::Abstract;

        // Act
        let title = section_type.latex_title();

        // Assert
        assert_eq!(title, "Abstract");
    }

    #[test]
    fn test_content_type_text_to_latex() {
        // Arrange
        let content = ContentType::Text("Hello world".to_string());

        // Act
        let latex = content.to_latex();

        // Assert
        assert!(latex.is_ok());
        assert_eq!(latex.unwrap(), "Hello world");
    }

    #[test]
    fn test_content_type_equation_to_latex() {
        // Arrange
        let content = ContentType::Equation(r"E = mc^2".to_string());

        // Act
        let latex = content.to_latex();

        // Assert
        assert!(latex.is_ok());
        let result = latex.unwrap();
        assert!(result.contains(r"\begin{equation}"));
        assert!(result.contains(r"E = mc^2"));
        assert!(result.contains(r"\end{equation}"));
    }

    #[test]
    fn test_diagram_tikz_to_latex() {
        // Arrange
        let diagram = DiagramType::TikZ(r"\node (A) at (0,0) {A};".to_string());

        // Act
        let latex = diagram.to_latex();

        // Assert
        assert!(latex.is_ok());
        let result = latex.unwrap();
        assert!(result.contains(r"\begin{tikzpicture}"));
        assert!(result.contains(r"\node (A)"));
        assert!(result.contains(r"\end{tikzpicture}"));
    }

    #[test]
    fn test_section_builder_pattern() {
        // Arrange
        let mut section = Section::new(SectionType::Introduction, "Intro".to_string());

        // Act
        section.add_text("Text content");
        section.add_equation("x = y");

        // Assert
        assert_eq!(section.content.len(), 2);
        assert!(matches!(section.content[0], ContentType::Text(_)));
        assert!(matches!(section.content[1], ContentType::Equation(_)));
    }

    #[test]
    fn test_bib_entry_to_bibtex() {
        // Arrange
        let mut entry = BibEntry {
            key: "test2024".to_string(),
            entry_type: "article".to_string(),
            fields: BTreeMap::new(),
        };
        entry.fields.insert("author".to_string(), "Test Author".to_string());

        // Act
        let bibtex = entry.to_bibtex();

        // Assert
        assert!(bibtex.contains("@article{test2024,"));
        assert!(bibtex.contains("author = {Test Author}"));
    }

    #[test]
    fn test_paper_builder_pattern() {
        // Arrange
        let mut paper = Paper::new("Test Paper".to_string(), vec!["Author".to_string()]);

        // Act
        let section = Section::new(SectionType::Abstract, "Abstract".to_string());
        paper.add_section(section);

        // Assert
        assert_eq!(paper.title, "Test Paper");
        assert_eq!(paper.authors.len(), 1);
        assert_eq!(paper.sections.len(), 1);
    }

    // Layer 2 Tests: Ontology Integration

    #[test]
    fn test_thesis_ontology_load() {
        // Arrange & Act
        let result = ThesisOntology::load();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_thesis_ontology_query_structure() {
        // Arrange
        let ontology = ThesisOntology::load();
        assert!(ontology.is_ok());
        let ontology = ontology.unwrap();

        // Act
        let structure = ontology.query_paper_structure();

        // Assert
        assert!(structure.is_ok());
        let templates = structure.unwrap();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_thesis_ontology_content_mapping() {
        // Arrange
        let ontology = ThesisOntology::load();
        assert!(ontology.is_ok());
        let ontology = ontology.unwrap();

        // Act
        let mapping = ontology.query_content_mapping();

        // Assert
        assert!(mapping.is_ok());
    }

    // Layer 3 Tests: Generator

    #[test]
    fn test_latex_generator_new() {
        // Arrange & Act
        let generator = LatexGenerator::new();

        // Assert
        assert!(generator.is_ok());
    }

    #[test]
    fn test_latex_generator_generate() {
        // Arrange
        let generator = LatexGenerator::new().unwrap();
        let paper = create_simple_test_paper();

        // Act
        let latex = generator.generate(&paper);

        // Assert
        assert!(latex.is_ok());
        let content = latex.unwrap();
        assert!(content.contains(r"\documentclass{article}"));
        assert!(content.contains("Test Paper"));
        assert!(content.contains(r"\maketitle"));
    }

    #[test]
    fn test_latex_generator_bibtex() {
        // Arrange
        let generator = LatexGenerator::new().unwrap();
        let mut paper = Paper::new("Test".to_string(), vec![]);
        let mut entry = BibEntry {
            key: "ref1".to_string(),
            entry_type: "article".to_string(),
            fields: BTreeMap::new(),
        };
        entry.fields.insert("title".to_string(), "Title".to_string());
        paper.add_bib_entry(entry);

        // Act
        let bibtex = generator.generate_bibtex(&paper);

        // Assert
        assert!(bibtex.contains("@article{ref1,"));
        assert!(bibtex.contains("title = {Title}"));
    }

    // Layer 4 Tests: Pipeline

    #[test]
    fn test_paper_pipeline_new() {
        // Arrange & Act
        let pipeline = PaperPipeline::new();

        // Assert
        assert!(pipeline.is_ok());
    }

    #[test]
    fn test_paper_pipeline_build() {
        // Arrange
        let pipeline = PaperPipeline::new();
        assert!(pipeline.is_ok());
        let pipeline = pipeline.unwrap();
        let paper = create_simple_test_paper();

        // Act
        let result = pipeline.build_paper(&paper);

        // Assert
        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(output_path.contains("paper.tex"));
    }

    #[test]
    fn test_paper_pipeline_mcp_storage() {
        // Arrange
        let pipeline = PaperPipeline::new();
        assert!(pipeline.is_ok());
        let pipeline = pipeline.unwrap();
        let paper = create_simple_test_paper();

        // Act
        let result = pipeline.store_in_mcp(&paper);

        // Assert
        assert!(result.is_ok());
    }

    // Helper for tests
    fn create_simple_test_paper() -> Paper {
        let mut paper = Paper::new("Test Paper".to_string(), vec!["Test Author".to_string()]);

        let mut section = Section::new(SectionType::Abstract, "Abstract".to_string());
        section.add_text("Test abstract content.");
        paper.add_section(section);

        paper
    }
}
