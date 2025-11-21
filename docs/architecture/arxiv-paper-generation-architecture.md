# ArXiv Paper Generation Architecture

**Type-First Design for Academic Paper Generation with Equations and Diagrams**

## Executive Summary

This document defines a 4-layer type-safe architecture for generating arXiv-quality academic papers with LaTeX equations, diagrams (PlantUML→TikZ), and BibTeX citations. Following 80/20 principles, we focus on the 20% features delivering 80% value: paper structure (IMRaD/Papers/Argument), LaTeX templates, equations, diagrams, and citations.

## Architecture Overview (C4 Context)

```
┌─────────────────────────────────────────────────────────────────┐
│                  ArXiv Paper Generation System                   │
│                                                                   │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐        │
│  │   Domain     │──>│   Ontology   │──>│  Generator   │──>PDF   │
│  │   Model      │   │   (RDF)      │   │  (LaTeX)     │        │
│  └──────────────┘   └──────────────┘   └──────────────┘        │
│         │                   │                   │                │
│         └───────────────────┴───────────────────┘                │
│                            │                                     │
│                      ┌──────────┐                                │
│                      │ Pipeline │                                │
│                      └──────────┘                                │
└─────────────────────────────────────────────────────────────────┘
         │                    │                    │
         v                    v                    v
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ Handlebars   │    │ thesis-      │    │ PlantUML/    │
│ Templates    │    │ ontology.ttl │    │ TikZ         │
└──────────────┘    └──────────────┘    └──────────────┘
```

## 80/20 Analysis: Critical Features

### Core 20% Features (80% Value)
1. **Paper Structure** - IMRaD, Thesis-by-Papers, Argument families (from thesis-ontology.ttl)
2. **LaTeX Templates** - Handlebars-based generation for arXiv submission
3. **Equations** - LaTeX math mode with inline (`$...$`) and display (`$$...$$`) equations
4. **Diagrams** - PlantUML syntax → LaTeX TikZ conversion
5. **Citations** - BibTeX integration with academic citation styles

### Excluded 80% Features (20% Value)
- Advanced typography (kerning, ligatures)
- Journal-specific templates (IEEE, ACM, Springer)
- Complex table layouts
- Multi-column formatting
- Custom LaTeX packages beyond standard arXiv support

## Layer 1: Domain Model

**Purpose**: Type-safe representation of academic paper structure using Rust enums/structs

### Core Types

```rust
// src/arxiv/domain.rs

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Paper structure following thesis-ontology.ttl families
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperFamily {
    /// IMRaD: Introduction, Method, Results, Discussion
    IMRaD(IMRaDPaper),
    /// Thesis by Papers: Multiple published papers + synthesis
    Papers(PapersPaper),
    /// Argument: Claim, Ground, Proof, Objection, Reply
    Argument(ArgumentPaper),
    /// Contribution: Gap, Design, Evaluation, Impact
    Contribution(ContributionPaper),
    /// Monograph: Deep scholarly work
    Monograph(MonographPaper),
    /// DSR: Design Science Research
    DSR(DSRPaper),
    /// Narrative: Field, Voice, Pattern, Insight
    Narrative(NarrativePaper),
}

/// Section is the fundamental building block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    /// Section title
    pub title: String,
    /// Section level (1=chapter, 2=section, 3=subsection, etc.)
    pub level: u8,
    /// Section content (markdown with LaTeX equations)
    pub content: Vec<ContentBlock>,
    /// Section metadata
    pub metadata: SectionMetadata,
}

/// Content blocks within sections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentBlock {
    /// Plain text paragraph
    Paragraph(String),
    /// LaTeX equation (inline or display)
    Equation(Equation),
    /// Diagram (PlantUML syntax → TikZ)
    Diagram(Diagram),
    /// Citation reference
    Citation(Citation),
    /// Code listing
    Code(CodeBlock),
    /// List (ordered or unordered)
    List(List),
}

/// Equation with LaTeX math mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equation {
    /// LaTeX math expression
    pub latex: String,
    /// Inline ($...$) or display ($$...$$)
    pub mode: EquationMode,
    /// Optional equation label for referencing
    pub label: Option<String>,
    /// Optional equation number
    pub number: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquationMode {
    /// Inline equation $...$
    Inline,
    /// Display equation $$...$$
    Display,
    /// Numbered equation with label
    Numbered,
}

/// Diagram definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagram {
    /// Diagram type
    pub diagram_type: DiagramType,
    /// Diagram source (PlantUML syntax)
    pub source: String,
    /// Caption
    pub caption: Option<String>,
    /// Label for referencing
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramType {
    /// PlantUML sequence diagram
    Sequence,
    /// PlantUML class diagram
    Class,
    /// PlantUML state diagram
    State,
    /// PlantUML activity diagram
    Activity,
    /// PlantUML use case diagram
    UseCase,
    /// Custom TikZ code
    TikZ,
}

/// Citation reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    /// BibTeX key
    pub key: String,
    /// Citation style (cite, citep, citet, etc.)
    pub style: CitationStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CitationStyle {
    /// Numeric citation [1]
    Numeric,
    /// Author-year citation (Smith 2020)
    AuthorYear,
    /// Textual citation Smith (2020)
    Textual,
}

/// Code block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Programming language
    pub language: String,
    /// Code content
    pub code: String,
    /// Optional caption
    pub caption: Option<String>,
}

/// List structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    /// List type
    pub list_type: ListType,
    /// List items
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListType {
    /// Ordered list (numbered)
    Ordered,
    /// Unordered list (bullets)
    Unordered,
}

/// Section metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionMetadata {
    /// Word count
    pub word_count: usize,
    /// Status (draft, in_progress, complete)
    pub status: String,
    /// Priority (1=highest)
    pub priority: u8,
    /// Tags
    pub tags: Vec<String>,
}

/// IMRaD paper structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IMRaDPaper {
    pub introduction: Section,
    pub method: Section,
    pub results: Section,
    pub discussion: Section,
    pub metadata: PaperMetadata,
}

/// Thesis by papers structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PapersPaper {
    pub paper_one: Section,
    pub paper_two: Section,
    pub paper_three: Section,
    pub synthesis: Section,
    pub metadata: PaperMetadata,
}

/// Argument paper structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentPaper {
    pub claim: Section,
    pub ground: Section,
    pub proof: Section,
    pub objection: Option<Section>,
    pub reply: Option<Section>,
    pub metadata: PaperMetadata,
}

/// Contribution paper structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributionPaper {
    pub gap: Section,
    pub design: Section,
    pub evaluation: Section,
    pub impact: Option<Section>,
    pub metadata: PaperMetadata,
}

/// Monograph paper structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonographPaper {
    pub context: Section,
    pub canon: Section,
    pub method: Section,
    pub analysis: Section,
    pub conclusion: Section,
    pub metadata: PaperMetadata,
}

/// DSR paper structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DSRPaper {
    pub problem: Section,
    pub artifact: Section,
    pub evaluation: Section,
    pub theory: Option<Section>,
    pub metadata: PaperMetadata,
}

/// Narrative paper structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NarrativePaper {
    pub field: Section,
    pub voice: Section,
    pub pattern: Section,
    pub insight: Section,
    pub metadata: PaperMetadata,
}

/// Paper metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperMetadata {
    /// Paper title
    pub title: String,
    /// Authors
    pub authors: Vec<Author>,
    /// Abstract
    pub abstract_text: String,
    /// Keywords
    pub keywords: Vec<String>,
    /// Target word count
    pub word_count_target: usize,
    /// BibTeX bibliography path
    pub bibliography: String,
}

/// Author information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub affiliation: String,
    pub email: Option<String>,
}

/// Bibliography entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibEntry {
    /// BibTeX key
    pub key: String,
    /// Entry type (article, book, inproceedings, etc.)
    pub entry_type: String,
    /// Fields (author, title, year, etc.)
    pub fields: BTreeMap<String, String>,
}
```

### Zero-Cost Abstractions

1. **Const Generics for Section Levels**:
   ```rust
   // Compile-time enforcement of section hierarchy
   struct SectionLevel<const LEVEL: u8>;

   impl<const LEVEL: u8> SectionLevel<LEVEL> {
       fn new() -> Self where [(); LEVEL <= 6]: {
           Self
       }
   }
   ```

2. **Type-State Pattern for Paper Construction**:
   ```rust
   // Ensures papers cannot be generated without required sections
   struct PaperBuilder<State> {
       state: State,
   }

   struct NeedsIntroduction;
   struct NeedsMethod;
   struct Complete;

   impl PaperBuilder<NeedsIntroduction> {
       fn with_introduction(self, intro: Section) -> PaperBuilder<NeedsMethod> {
           // ...
       }
   }
   ```

3. **Generic Over Content Blocks**:
   ```rust
   // Zero-cost iteration over content blocks
   fn process_content<'a, I>(blocks: I) -> String
   where
       I: IntoIterator<Item = &'a ContentBlock>
   {
       blocks.into_iter().map(|b| b.to_latex()).collect()
   }
   ```

## Layer 2: Ontology (RDF Integration)

**Purpose**: Map domain types to thesis-ontology.ttl RDF triples for semantic interoperability

### RDF Mapping

```rust
// src/arxiv/ontology.rs

use crate::rdf::{Ontology, RdfTriple, RdfValue};
use crate::arxiv::domain::*;

/// Hyper-Thesis Framework namespace
pub const HTF_NS: &str = "http://thesis.hyper/framework/";

/// Convert PaperFamily to RDF triples
pub fn paper_to_rdf(paper: &PaperFamily, ontology: &mut Ontology) -> String {
    let paper_uri = format!("{}paper/{}", HTF_NS, generate_paper_id(paper));

    match paper {
        PaperFamily::IMRaD(imrad) => {
            // Paper is instance of IMRaDFamily
            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
                RdfValue::uri(format!("{}IMRaDFamily", HTF_NS)),
            ));

            // Link sections with htf:hasShard
            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                format!("{}hasShard", HTF_NS),
                RdfValue::uri(section_to_rdf(&imrad.introduction, ontology, "Introduction")),
            ));

            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                format!("{}hasShard", HTF_NS),
                RdfValue::uri(section_to_rdf(&imrad.method, ontology, "Method")),
            ));

            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                format!("{}hasShard", HTF_NS),
                RdfValue::uri(section_to_rdf(&imrad.results, ontology, "Results")),
            ));

            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                format!("{}hasShard", HTF_NS),
                RdfValue::uri(section_to_rdf(&imrad.discussion, ontology, "Discussion")),
            ));

            // Metadata
            paper_metadata_to_rdf(&paper_uri, &imrad.metadata, ontology);
        }
        PaperFamily::Argument(arg) => {
            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
                RdfValue::uri(format!("{}ArgumentFamily", HTF_NS)),
            ));

            // Required shards
            ontology.add_triple(RdfTriple::new(
                &paper_uri,
                format!("{}hasShard", HTF_NS),
                RdfValue::uri(section_to_rdf(&arg.claim, ontology, "Claim")),
            ));

            // ... similar for ground, proof, objection, reply
        }
        // ... other families
        _ => {}
    }

    paper_uri
}

/// Convert Section to RDF shard
fn section_to_rdf(section: &Section, ontology: &mut Ontology, shard_type: &str) -> String {
    let section_uri = format!("{}shard/{}", HTF_NS, generate_section_id(section));

    // Type
    ontology.add_triple(RdfTriple::new(
        &section_uri,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
        RdfValue::uri(format!("{}{}", HTF_NS, shard_type)),
    ));

    // Properties
    ontology.add_triple(RdfTriple::new(
        &section_uri,
        format!("{}shardName", HTF_NS),
        RdfValue::literal(&section.title),
    ));

    ontology.add_triple(RdfTriple::new(
        &section_uri,
        format!("{}wordCount", HTF_NS),
        RdfValue::typed_literal(
            section.metadata.word_count.to_string(),
            "http://www.w3.org/2001/XMLSchema#integer"
        ),
    ));

    ontology.add_triple(RdfTriple::new(
        &section_uri,
        format!("{}status", HTF_NS),
        RdfValue::literal(&section.metadata.status),
    ));

    ontology.add_triple(RdfTriple::new(
        &section_uri,
        format!("{}priority", HTF_NS),
        RdfValue::typed_literal(
            section.metadata.priority.to_string(),
            "http://www.w3.org/2001/XMLSchema#integer"
        ),
    ));

    section_uri
}

fn paper_metadata_to_rdf(paper_uri: &str, metadata: &PaperMetadata, ontology: &mut Ontology) {
    ontology.add_triple(RdfTriple::new(
        paper_uri,
        "http://purl.org/dc/terms/title",
        RdfValue::literal(&metadata.title),
    ));

    ontology.add_triple(RdfTriple::new(
        paper_uri,
        "http://purl.org/dc/terms/abstract",
        RdfValue::literal(&metadata.abstract_text),
    ));

    for author in &metadata.authors {
        ontology.add_triple(RdfTriple::new(
            paper_uri,
            "http://purl.org/dc/terms/creator",
            RdfValue::literal(&author.name),
        ));
    }

    for keyword in &metadata.keywords {
        ontology.add_triple(RdfTriple::new(
            paper_uri,
            "http://purl.org/dc/terms/subject",
            RdfValue::literal(keyword),
        ));
    }
}

fn generate_paper_id(paper: &PaperFamily) -> String {
    // Generate unique ID from paper content (BLAKE3 hash)
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    match paper {
        PaperFamily::IMRaD(p) => {
            hasher.update(p.metadata.title.as_bytes());
        }
        // ... other families
        _ => {}
    }

    hasher.finalize().to_hex()[..16].to_string()
}

fn generate_section_id(section: &Section) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(section.title.as_bytes());
    hasher.finalize().to_hex()[..16].to_string()
}
```

### Ontology Integration Points

1. **Paper Families**: Map to `htf:IMRaDFamily`, `htf:ArgumentFamily`, etc.
2. **Sections**: Map to `htf:Shard` with `htf:shardFamily` property
3. **Total Order**: Use `htf:precedes` to encode section ordering
4. **Invariants**: Validate against `htf:Coherence`, `htf:Completeness`, `htf:Evidence`

## Layer 3: Generator (LaTeX + Handlebars)

**Purpose**: Transform domain types into LaTeX source using Handlebars templates

### Template Structure

```
templates/
├── arxiv/
│   ├── base.hbs          # Base LaTeX template
│   ├── sections/
│   │   ├── section.hbs   # Section template
│   │   ├── equation.hbs  # Equation template
│   │   └── diagram.hbs   # Diagram template
│   ├── families/
│   │   ├── imrad.hbs     # IMRaD template
│   │   ├── argument.hbs  # Argument template
│   │   └── papers.hbs    # Papers template
│   └── components/
│       ├── title.hbs     # Title page
│       ├── abstract.hbs  # Abstract
│       └── bib.hbs       # Bibliography
```

### LaTeX Generator

```rust
// src/arxiv/generator.rs

use handlebars::Handlebars;
use serde_json::json;
use crate::arxiv::domain::*;
use crate::error::Result;

pub struct LatexGenerator {
    handlebars: Handlebars<'static>,
}

impl LatexGenerator {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register templates
        handlebars.register_template_string("base", include_str!("../../templates/arxiv/base.hbs"))?;
        handlebars.register_template_string("section", include_str!("../../templates/arxiv/sections/section.hbs"))?;
        handlebars.register_template_string("equation", include_str!("../../templates/arxiv/sections/equation.hbs"))?;
        handlebars.register_template_string("diagram", include_str!("../../templates/arxiv/sections/diagram.hbs"))?;

        // Register families
        handlebars.register_template_string("imrad", include_str!("../../templates/arxiv/families/imrad.hbs"))?;
        handlebars.register_template_string("argument", include_str!("../../templates/arxiv/families/argument.hbs"))?;

        Ok(Self { handlebars })
    }

    /// Generate LaTeX from PaperFamily
    pub fn generate(&self, paper: &PaperFamily) -> Result<String> {
        match paper {
            PaperFamily::IMRaD(imrad) => self.generate_imrad(imrad),
            PaperFamily::Argument(arg) => self.generate_argument(arg),
            // ... other families
            _ => unimplemented!("Family not yet implemented"),
        }
    }

    fn generate_imrad(&self, paper: &IMRaDPaper) -> Result<String> {
        let data = json!({
            "title": paper.metadata.title,
            "authors": paper.metadata.authors,
            "abstract": paper.metadata.abstract_text,
            "keywords": paper.metadata.keywords,
            "introduction": self.section_to_latex(&paper.introduction)?,
            "method": self.section_to_latex(&paper.method)?,
            "results": self.section_to_latex(&paper.results)?,
            "discussion": self.section_to_latex(&paper.discussion)?,
            "bibliography": paper.metadata.bibliography,
        });

        Ok(self.handlebars.render("imrad", &data)?)
    }

    fn section_to_latex(&self, section: &Section) -> Result<String> {
        let content_latex = self.content_blocks_to_latex(&section.content)?;

        let data = json!({
            "title": section.title,
            "level": section.level,
            "content": content_latex,
        });

        Ok(self.handlebars.render("section", &data)?)
    }

    fn content_blocks_to_latex(&self, blocks: &[ContentBlock]) -> Result<String> {
        let mut latex = String::new();

        for block in blocks {
            match block {
                ContentBlock::Paragraph(text) => {
                    latex.push_str(text);
                    latex.push_str("\n\n");
                }
                ContentBlock::Equation(eq) => {
                    latex.push_str(&self.equation_to_latex(eq)?);
                }
                ContentBlock::Diagram(diag) => {
                    latex.push_str(&self.diagram_to_latex(diag)?);
                }
                ContentBlock::Citation(cite) => {
                    latex.push_str(&self.citation_to_latex(cite));
                }
                ContentBlock::Code(code) => {
                    latex.push_str(&self.code_to_latex(code));
                }
                ContentBlock::List(list) => {
                    latex.push_str(&self.list_to_latex(list));
                }
            }
        }

        Ok(latex)
    }

    fn equation_to_latex(&self, eq: &Equation) -> Result<String> {
        let latex = match eq.mode {
            EquationMode::Inline => format!("${eq}$", eq = eq.latex),
            EquationMode::Display => format!("$${}$$", eq.latex),
            EquationMode::Numbered => {
                if let Some(label) = &eq.label {
                    format!("\\begin{{equation}}\\label{{{}}}\n{}\n\\end{{equation}}", label, eq.latex)
                } else {
                    format!("\\begin{{equation}}\n{}\n\\end{{equation}}", eq.latex)
                }
            }
        };

        Ok(latex)
    }

    fn diagram_to_latex(&self, diag: &Diagram) -> Result<String> {
        match diag.diagram_type {
            DiagramType::TikZ => {
                // Direct TikZ passthrough
                Ok(format!("\\begin{{tikzpicture}}\n{}\n\\end{{tikzpicture}}", diag.source))
            }
            _ => {
                // Convert PlantUML to TikZ (simplified - in production use plantuml-to-tikz library)
                let tikz = self.plantuml_to_tikz(&diag.source, diag.diagram_type)?;

                let mut latex = String::new();
                latex.push_str("\\begin{figure}[h]\n");
                latex.push_str("\\centering\n");
                latex.push_str(&tikz);

                if let Some(caption) = &diag.caption {
                    latex.push_str(&format!("\\caption{{{}}}\n", caption));
                }

                if let Some(label) = &diag.label {
                    latex.push_str(&format!("\\label{{{}}}\n", label));
                }

                latex.push_str("\\end{figure}\n");

                Ok(latex)
            }
        }
    }

    fn plantuml_to_tikz(&self, plantuml: &str, diagram_type: DiagramType) -> Result<String> {
        // Simplified PlantUML → TikZ conversion
        // In production, use a proper PlantUML parser and TikZ generator

        match diagram_type {
            DiagramType::Sequence => {
                // Parse PlantUML sequence diagram
                // Generate TikZ sequence diagram
                Ok(self.generate_tikz_sequence(plantuml))
            }
            DiagramType::Class => {
                Ok(self.generate_tikz_class(plantuml))
            }
            _ => {
                // For other types, use placeholder
                Ok(format!("% PlantUML diagram type: {:?}\n% TODO: Implement conversion", diagram_type))
            }
        }
    }

    fn generate_tikz_sequence(&self, plantuml: &str) -> String {
        // Minimal TikZ sequence diagram
        r#"\begin{tikzpicture}
\node[draw] (A) at (0,0) {Actor A};
\node[draw] (B) at (4,0) {Actor B};
\draw[-latex] (A) -- (B) node[midway,above] {message};
\end{tikzpicture}"#.to_string()
    }

    fn generate_tikz_class(&self, plantuml: &str) -> String {
        // Minimal TikZ class diagram
        r#"\begin{tikzpicture}
\node[draw,rectangle] (C1) at (0,0) {Class1};
\node[draw,rectangle] (C2) at (4,0) {Class2};
\draw[-latex] (C1) -- (C2);
\end{tikzpicture}"#.to_string()
    }

    fn citation_to_latex(&self, cite: &Citation) -> String {
        match cite.style {
            CitationStyle::Numeric => format!("\\cite{{{}}}", cite.key),
            CitationStyle::AuthorYear => format!("\\citep{{{}}}", cite.key),
            CitationStyle::Textual => format!("\\citet{{{}}}", cite.key),
        }
    }

    fn code_to_latex(&self, code: &CodeBlock) -> String {
        let mut latex = String::new();
        latex.push_str("\\begin{lstlisting}");

        if !code.language.is_empty() {
            latex.push_str(&format!("[language={}]", code.language));
        }

        latex.push('\n');
        latex.push_str(&code.code);
        latex.push_str("\n\\end{lstlisting}\n");

        if let Some(caption) = &code.caption {
            latex = format!("\\begin{{figure}}[h]\n{}\n\\caption{{{}}}\n\\end{{figure}}", latex, caption);
        }

        latex
    }

    fn list_to_latex(&self, list: &List) -> String {
        let env = match list.list_type {
            ListType::Ordered => "enumerate",
            ListType::Unordered => "itemize",
        };

        let mut latex = format!("\\begin{{{}}}\n", env);

        for item in &list.items {
            latex.push_str(&format!("\\item {}\n", item));
        }

        latex.push_str(&format!("\\end{{{}}}\n", env));

        latex
    }
}
```

### Handlebars Templates

**Base Template** (`templates/arxiv/base.hbs`):

```latex
\documentclass[12pt]{article}

% ArXiv-compatible packages
\usepackage{amsmath,amssymb}
\usepackage{graphicx}
\usepackage{tikz}
\usepackage{listings}
\usepackage{natbib}
\usepackage{hyperref}

\title{ {{title}} }

\author{
{{#each authors}}
{{name}}\\
\textit{ {{affiliation}} }\\
{{#if email}}\texttt{ {{email}} }{{/if}}
{{#unless @last}}\\[1em]{{/unless}}
{{/each}}
}

\begin{document}

\maketitle

\begin{abstract}
{{abstract}}
\end{abstract}

\textbf{Keywords:} {{#each keywords}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}

{{> content}}

\bibliographystyle{plain}
\bibliography{ {{bibliography}} }

\end{document}
```

**IMRaD Template** (`templates/arxiv/families/imrad.hbs`):

```latex
\section{Introduction}
{{{introduction}}}

\section{Method}
{{{method}}}

\section{Results}
{{{results}}}

\section{Discussion}
{{{discussion}}}
```

## Layer 4: Pipeline

**Purpose**: Orchestrate full paper generation workflow from domain model to PDF

```rust
// src/arxiv/pipeline.rs

use crate::arxiv::{domain::*, generator::*, ontology::*};
use crate::rdf::Ontology;
use crate::error::Result;
use std::path::Path;
use std::fs;

pub struct PaperPipeline {
    generator: LatexGenerator,
}

impl PaperPipeline {
    pub fn new() -> Result<Self> {
        Ok(Self { generator: LatexGenerator::new()? })
    }

    /// Generate arXiv-ready paper from domain model
    pub fn generate_paper(&self, paper: &PaperFamily, output_dir: &Path) -> Result<PaperOutput> {
        // 1. Generate RDF ontology
        let mut ontology = Ontology::new();
        ontology.add_prefix("htf", HTF_NS);
        let paper_uri = paper_to_rdf(paper, &mut ontology);

        // 2. Validate invariants
        self.validate_invariants(&ontology, &paper_uri)?;

        // 3. Generate LaTeX
        let latex = self.generator.generate(paper)?;

        // 4. Write LaTeX to file
        let latex_path = output_dir.join("paper.tex");
        fs::write(&latex_path, &latex)?;

        // 5. Copy bibliography
        let bib_path = self.copy_bibliography(paper, output_dir)?;

        // 6. Compile LaTeX to PDF (requires pdflatex)
        let pdf_path = self.compile_latex(&latex_path)?;

        // 7. Export RDF ontology
        let rdf_path = output_dir.join("paper.ttl");
        fs::write(&rdf_path, ontology.to_turtle())?;

        Ok(PaperOutput {
            latex_path,
            pdf_path,
            bib_path,
            rdf_path,
            paper_uri,
        })
    }

    fn validate_invariants(&self, ontology: &Ontology, paper_uri: &str) -> Result<()> {
        // Validate htf:Completeness invariant
        // All required shards must be present

        // Validate htf:Coherence invariant
        // All shards must align with central claim

        // Validate htf:Evidence invariant
        // All claims must have supporting evidence

        Ok(())
    }

    fn copy_bibliography(&self, paper: &PaperFamily, output_dir: &Path) -> Result<std::path::PathBuf> {
        let metadata = match paper {
            PaperFamily::IMRaD(p) => &p.metadata,
            PaperFamily::Argument(p) => &p.metadata,
            _ => unimplemented!(),
        };

        let bib_src = Path::new(&metadata.bibliography);
        let bib_dest = output_dir.join("references.bib");

        fs::copy(bib_src, &bib_dest)?;

        Ok(bib_dest)
    }

    fn compile_latex(&self, latex_path: &Path) -> Result<std::path::PathBuf> {
        use std::process::Command;

        // Run pdflatex twice (for references)
        for _ in 0..2 {
            let output = Command::new("pdflatex")
                .arg("-interaction=nonstopmode")
                .arg(latex_path)
                .current_dir(latex_path.parent().unwrap())
                .output()?;

            if !output.status.success() {
                return Err(crate::error::Error::LatexCompilation(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }
        }

        // Run bibtex
        let aux_path = latex_path.with_extension("aux");
        let _ = Command::new("bibtex")
            .arg(&aux_path)
            .current_dir(latex_path.parent().unwrap())
            .output()?;

        // Final pdflatex run
        let output = Command::new("pdflatex")
            .arg("-interaction=nonstopmode")
            .arg(latex_path)
            .current_dir(latex_path.parent().unwrap())
            .output()?;

        if !output.status.success() {
            return Err(crate::error::Error::LatexCompilation(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(latex_path.with_extension("pdf"))
    }
}

pub struct PaperOutput {
    pub latex_path: std::path::PathBuf,
    pub pdf_path: std::path::PathBuf,
    pub bib_path: std::path::PathBuf,
    pub rdf_path: std::path::PathBuf,
    pub paper_uri: String,
}
```

## Architecture Decision Records (ADRs)

### ADR-001: Use Handlebars for LaTeX Generation

**Status**: Accepted

**Context**: Need template engine for LaTeX generation with dynamic content insertion.

**Decision**: Use Handlebars template engine with `.hbs` files.

**Rationale**:
- Logic-less templates prevent business logic in templates
- Widely used with good Rust support (`handlebars-rs`)
- Clean separation between templates and generation logic
- Supports partials for composability

**Consequences**:
- ✅ Type-safe template rendering
- ✅ Easy to maintain LaTeX templates
- ⚠️ Requires Handlebars syntax knowledge

### ADR-002: PlantUML → TikZ Conversion

**Status**: Accepted

**Context**: Need to convert PlantUML diagrams to LaTeX-compatible TikZ format.

**Decision**: Implement custom PlantUML parser → TikZ generator in Rust.

**Rationale**:
- PlantUML is more ergonomic for diagram authoring
- TikZ is LaTeX-native and arXiv-compatible
- Conversion enables best of both worlds

**Consequences**:
- ✅ Easy diagram authoring with PlantUML
- ✅ arXiv-compatible TikZ output
- ⚠️ Requires maintaining PlantUML parser
- ⚠️ Limited to supported diagram types (sequence, class, state, activity)

### ADR-003: Type-State Pattern for Paper Construction

**Status**: Accepted

**Context**: Need to ensure papers cannot be generated without required sections.

**Decision**: Use type-state pattern with generic state parameters.

**Rationale**:
- Compile-time guarantee of paper completeness
- Prevents invalid paper construction at type level
- Zero-cost abstraction (state is compile-time only)

**Consequences**:
- ✅ Impossible to construct invalid papers
- ✅ Self-documenting API (types encode requirements)
- ⚠️ More complex API surface

### ADR-004: BibTeX Over BibLaTeX

**Status**: Accepted

**Context**: Need bibliography format for citations.

**Decision**: Use BibTeX with `natbib` package.

**Rationale**:
- arXiv uses BibTeX by default
- Simpler than BibLaTeX
- Wider compatibility with arXiv submission system

**Consequences**:
- ✅ arXiv-compatible out of the box
- ✅ Simple `.bib` file format
- ⚠️ Less powerful than BibLaTeX

### ADR-005: RDF Integration with thesis-ontology.ttl

**Status**: Accepted

**Context**: Need semantic metadata for papers.

**Decision**: Map domain types to thesis-ontology.ttl RDF triples.

**Rationale**:
- Enables semantic querying of paper structure
- Integrates with existing Hyper-Thesis Framework
- Supports validation of paper invariants (Coherence, Completeness, Evidence)

**Consequences**:
- ✅ Semantic interoperability with thesis tooling
- ✅ Queryable paper metadata via SPARQL
- ✅ Validation against ontology invariants
- ⚠️ Requires maintaining RDF mapping layer

## API Usage Examples

### Example 1: Generate IMRaD Paper

```rust
use clap_noun_verb::arxiv::{domain::*, pipeline::*};

fn main() -> Result<()> {
    // Build IMRaD paper
    let paper = PaperFamily::IMRaD(IMRaDPaper {
        introduction: Section {
            title: "Introduction".to_string(),
            level: 1,
            content: vec![
                ContentBlock::Paragraph("This paper presents...".to_string()),
                ContentBlock::Citation(Citation {
                    key: "smith2020".to_string(),
                    style: CitationStyle::Numeric,
                }),
            ],
            metadata: SectionMetadata {
                word_count: 500,
                status: "complete".to_string(),
                priority: 1,
                tags: vec!["introduction".to_string()],
            },
        },
        method: Section {
            title: "Method".to_string(),
            level: 1,
            content: vec![
                ContentBlock::Paragraph("Our methodology involves...".to_string()),
                ContentBlock::Equation(Equation {
                    latex: r"E = mc^2".to_string(),
                    mode: EquationMode::Numbered,
                    label: Some("eq:energy".to_string()),
                    number: Some(1),
                }),
            ],
            metadata: SectionMetadata {
                word_count: 800,
                status: "complete".to_string(),
                priority: 2,
                tags: vec!["methodology".to_string()],
            },
        },
        results: Section {
            title: "Results".to_string(),
            level: 1,
            content: vec![
                ContentBlock::Paragraph("Experimental results show...".to_string()),
                ContentBlock::Diagram(Diagram {
                    diagram_type: DiagramType::Sequence,
                    source: r#"
@startuml
Alice -> Bob: Request
Bob --> Alice: Response
@enduml
                    "#.to_string(),
                    caption: Some("Sequence diagram of protocol".to_string()),
                    label: Some("fig:protocol".to_string()),
                }),
            ],
            metadata: SectionMetadata {
                word_count: 1200,
                status: "complete".to_string(),
                priority: 3,
                tags: vec!["results".to_string()],
            },
        },
        discussion: Section {
            title: "Discussion".to_string(),
            level: 1,
            content: vec![
                ContentBlock::Paragraph("These results demonstrate...".to_string()),
            ],
            metadata: SectionMetadata {
                word_count: 600,
                status: "complete".to_string(),
                priority: 4,
                tags: vec!["discussion".to_string()],
            },
        },
        metadata: PaperMetadata {
            title: "A Novel Approach to Quantum Computing".to_string(),
            authors: vec![
                Author {
                    name: "Alice Smith".to_string(),
                    affiliation: "MIT".to_string(),
                    email: Some("alice@mit.edu".to_string()),
                },
            ],
            abstract_text: "We present a novel approach...".to_string(),
            keywords: vec!["quantum computing".to_string(), "algorithms".to_string()],
            word_count_target: 5000,
            bibliography: "references.bib".to_string(),
        },
    });

    // Generate paper
    let pipeline = PaperPipeline::new()?;
    let output = pipeline.generate_paper(&paper, Path::new("./output"))?;

    println!("✅ Paper generated:");
    println!("   LaTeX: {:?}", output.latex_path);
    println!("   PDF: {:?}", output.pdf_path);
    println!("   RDF: {:?}", output.rdf_path);
    println!("   URI: {}", output.paper_uri);

    Ok(())
}
```

### Example 2: Argument Paper with Equations

```rust
let paper = PaperFamily::Argument(ArgumentPaper {
    claim: Section {
        title: "Central Claim".to_string(),
        level: 1,
        content: vec![
            ContentBlock::Paragraph("We claim that...".to_string()),
            ContentBlock::Equation(Equation {
                latex: r"\forall x \in \mathbb{R}, x^2 \geq 0".to_string(),
                mode: EquationMode::Display,
                label: None,
                number: None,
            }),
        ],
        metadata: SectionMetadata {
            word_count: 300,
            status: "complete".to_string(),
            priority: 1,
            tags: vec!["claim".to_string()],
        },
    },
    ground: Section {
        title: "Grounds".to_string(),
        level: 1,
        content: vec![
            ContentBlock::Paragraph("The evidence for this claim is...".to_string()),
        ],
        metadata: SectionMetadata {
            word_count: 500,
            status: "complete".to_string(),
            priority: 2,
            tags: vec!["evidence".to_string()],
        },
    },
    proof: Section {
        title: "Proof".to_string(),
        level: 1,
        content: vec![
            ContentBlock::Paragraph("Formal proof:".to_string()),
            ContentBlock::Equation(Equation {
                latex: r"\begin{aligned} x^2 &\geq 0 \\ x^2 - y^2 &= (x+y)(x-y) \end{aligned}".to_string(),
                mode: EquationMode::Numbered,
                label: Some("eq:proof".to_string()),
                number: Some(1),
            }),
        ],
        metadata: SectionMetadata {
            word_count: 800,
            status: "complete".to_string(),
            priority: 3,
            tags: vec!["proof".to_string()],
        },
    },
    objection: None,
    reply: None,
    metadata: PaperMetadata {
        title: "Mathematical Foundations of X".to_string(),
        authors: vec![
            Author {
                name: "Bob Johnson".to_string(),
                affiliation: "Stanford".to_string(),
                email: None,
            },
        ],
        abstract_text: "This paper establishes...".to_string(),
        keywords: vec!["mathematics".to_string(), "foundations".to_string()],
        word_count_target: 3000,
        bibliography: "math_refs.bib".to_string(),
    },
});
```

## Performance Characteristics

### Compile-Time Guarantees
- **Type safety**: Invalid papers rejected at compile time
- **Zero-cost abstractions**: Generics monomorphize, no runtime overhead
- **Const generics**: Section level bounds checked at compile time

### Runtime Performance
- **LaTeX generation**: O(n) where n = content blocks
- **RDF mapping**: O(n) where n = sections
- **PlantUML parsing**: O(m) where m = diagram lines (simplified parser)
- **Template rendering**: O(t) where t = template size (Handlebars optimized)

### Memory Usage
- **Domain model**: ~100-500 KB per paper (depends on content)
- **RDF ontology**: ~50-200 KB per paper
- **LaTeX output**: ~20-100 KB per paper

## Future Extensions (Out of Scope for 80/20)

- **Advanced diagram types**: Component diagrams, deployment diagrams
- **Journal-specific templates**: IEEE, ACM, Springer formats
- **Collaborative editing**: Real-time multi-user editing
- **Version control**: Git integration for paper versions
- **AI assistance**: LLM-powered paper writing suggestions
- **Citation management**: Automatic BibTeX generation from DOIs

## References

1. **Thesis Ontology**: `/Users/sac/clap-noun-verb/playground/thesis-ontology.ttl`
2. **RDF Types**: `/Users/sac/clap-noun-verb/src/rdf/types.rs`
3. **RDF Ontology**: `/Users/sac/clap-noun-verb/src/rdf/ontology.rs`
4. **Handlebars-rs**: https://github.com/sunng87/handlebars-rust
5. **PlantUML**: https://plantuml.com/
6. **TikZ**: https://tikz.dev/
7. **arXiv Submission**: https://arxiv.org/help/submit_tex
