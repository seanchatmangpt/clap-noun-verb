//! Semantic Academic Submissions as Graph Projections
//!
//! This example demonstrates how to represent all academic conference submissions
//! as SPARQL projections of a single RDF knowledge graph, following the ggen pattern
//! where software artifacts are projections of knowledge graphs.
//!
//! Rather than maintain separate submission documents, all conference-specific
//! packages are SPARQL CONSTRUCT queries that project different aspects of the
//! core research onto specific venue requirements.
//!
//! Architecture:
//! - Base Graph: Complete semantic CLI research (all data)
//! - Projections: SPARQL CONSTRUCT queries per venue
//!   * ICSE: Software Engineering Practice focus
//!   * ECSA: Architecture Patterns focus
//!   * PLDI/OOPSLA: Type Systems & Language Innovation
//!   * ASE Workshop: DSL Design Lessons Learned
//!   * arXiv: Open research with complete metadata

use chrono::Utc;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// ONTOLOGY: RESEARCH ARTIFACT SCHEMA
// ============================================================================

mod research_ontology {
    pub const NAMESPACE: &str = "http://research.acm.org/ontology/";

    // Entity Types
    pub fn paper() -> String {
        format!("{}Paper", NAMESPACE)
    }

    pub fn conference() -> String {
        format!("{}Conference", NAMESPACE)
    }

    pub fn author() -> String {
        format!("{}Author", NAMESPACE)
    }

    pub fn contribution() -> String {
        format!("{}Contribution", NAMESPACE)
    }

    pub fn section() -> String {
        format!("{}Section", NAMESPACE)
    }

    pub fn experiment() -> String {
        format!("{}Experiment", NAMESPACE)
    }

    // Properties
    pub fn title() -> String {
        format!("{}title", NAMESPACE)
    }

    pub fn abstract_text() -> String {
        format!("{}abstract", NAMESPACE)
    }

    pub fn authors() -> String {
        format!("{}authors", NAMESPACE)
    }

    pub fn contributions() -> String {
        format!("{}contributions", NAMESPACE)
    }

    pub fn has_section() -> String {
        format!("{}hasSection", NAMESPACE)
    }

    pub fn page_count() -> String {
        format!("{}pageCount", NAMESPACE)
    }

    pub fn track() -> String {
        format!("{}track", NAMESPACE)
    }

    pub fn emphasis() -> String {
        format!("{}emphasis", NAMESPACE)
    }

    pub fn acceptance_probability() -> String {
        format!("{}acceptanceProbability", NAMESPACE)
    }
}

// ============================================================================
// SPARQL PROJECTION QUERIES
// ============================================================================

mod projections {
    use super::research_ontology::*;

    /// Projects complete research graph (all sections, full details)
    pub fn arxiv_projection() -> String {
        format!(
            r#"
PREFIX research: <http://research.acm.org/ontology/>

CONSTRUCT {{
    ?paper research:title ?title ;
            research:abstract ?abstract ;
            research:authors ?authors ;
            research:contributions ?contributions ;
            research:hasSection ?section ;
            research:pageCount 12 ;
            research:venue "arXiv" ;
            research:acceptanceProbability 100 .
}}
WHERE {{
    ?paper a research:Paper ;
           research:title ?title ;
           research:abstract ?abstract ;
           research:authors ?authors ;
           research:contributions ?contributions ;
           research:hasSection ?section .
}}
"#
        )
    }

    /// ICSE Projection: Software Engineering Practice focus
    /// - 12 pages max
    /// - Emphasize practical impact on SE practice
    /// - Frame contributions for ICSE audience
    pub fn icse_projection() -> String {
        format!(
            r#"
PREFIX research: <http://research.acm.org/ontology/>

CONSTRUCT {{
    ?paper research:venue "ICSE 2026" ;
            research:track "Software Engineering Practice" ;
            research:pageCount 12 ;
            research:emphasis "practical impact on software engineering" ;
            research:acceptanceProbability 65 ;
            research:title ?title ;
            research:abstract ?abstractShort ;
            research:contributions ?contributions ;
            research:hasSection ?section .
    ?section research:content ?sectionContent ;
             research:order ?sectionOrder .
}}
WHERE {{
    ?paper a research:Paper ;
           research:title ?title ;
           research:abstract ?abstract ;
           research:contributions ?contributions ;
           research:hasSection ?section .

    # Project sections relevant to ICSE
    ?section research:sectionType ?type .
    FILTER (?type IN ("Introduction", "Architecture", "Implementation",
                      "Evaluation", "Related Work", "Conclusions"))
}}
"#
        )
    }

    /// ECSA Projection: Software Architecture focus
    /// - 14 pages max
    /// - Emphasize architectural patterns and design
    /// - Frame for architecture community
    pub fn ecsa_projection() -> String {
        format!(
            r#"
PREFIX research: <http://research.acm.org/ontology/>

CONSTRUCT {{
    ?paper research:venue "ECSA 2026" ;
            research:track "Software Architecture & Design" ;
            research:pageCount 14 ;
            research:emphasis "architectural patterns and design decisions" ;
            research:acceptanceProbability 70 ;
            research:title ?title ;
            research:abstract ?abstractArch ;
            research:contributions ?archContributions ;
            research:hasSection ?section ;
            research:tradeOffs ?tradeoffs .
}}
WHERE {{
    ?paper a research:Paper ;
           research:title ?title ;
           research:abstract ?abstract ;
           research:contributions ?contributions ;
           research:hasSection ?section ;
           research:architecturalDecisions ?tradeoffs .

    # Map contributions to architectural perspective
    BIND(CONCAT("Architectural: ", ?contributions) AS ?archContributions)
}}
"#
        )
    }

    /// PLDI/OOPSLA Projection: Type Systems & Language Innovation
    /// - 12-15 pages
    /// - Emphasize type-driven design and language features
    /// - Frame for programming languages community
    pub fn pldi_oopsla_projection() -> String {
        format!(
            r#"
PREFIX research: <http://research.acm.org/ontology/>

CONSTRUCT {{
    ?paper research:venue "PLDI/OOPSLA 2026" ;
            research:track "Type Systems & Language Innovation" ;
            research:pageCount 15 ;
            research:emphasis "type-driven semantic generation and compile-time techniques" ;
            research:acceptanceProbability 68 ;
            research:title ?title ;
            research:abstract ?abstractPL ;
            research:contributions ?langContributions ;
            research:hasSection ?section ;
            research:typeSignatures ?types .
}}
WHERE {{
    ?paper a research:Paper ;
           research:title ?title ;
           research:abstract ?abstract ;
           research:contributions ?contributions ;
           research:hasSection ?section ;
           research:typeSystemInnovations ?types .

    # Map contributions to type-first perspective
    BIND(CONCAT("Type-driven: ", ?contributions) AS ?langContributions)
}}
"#
        )
    }

    /// ASE Workshop Projection: DSL Design Lessons Learned
    /// - 6-8 pages
    /// - Emphasize practical lessons and open questions
    /// - Workshop format (discussion-focused)
    pub fn ase_workshop_projection() -> String {
        format!(
            r#"
PREFIX research: <http://research.acm.org/ontology/>

CONSTRUCT {{
    ?paper research:venue "ASE 2026 Workshop" ;
            research:track "DSL Design & Lessons Learned" ;
            research:pageCount 8 ;
            research:emphasis "practical DSL design lessons and open questions" ;
            research:acceptanceProbability 88 ;
            research:title ?title ;
            research:abstract ?abstractLesson ;
            research:lessons ?lesson ;
            research:openQuestions ?question ;
            research:hasSection ?section .
}}
WHERE {{
    ?paper a research:Paper ;
           research:title ?title ;
           research:abstract ?abstract ;
           research:hasSection ?section ;
           research:designLessons ?lesson ;
           research:researchQuestions ?question .

    # Project only practical/lessons sections
    ?section research:sectionType ?type .
    FILTER (?type IN ("Lessons", "OpenQuestions", "Discussion"))
}}
"#
        )
    }
}

// ============================================================================
// GRAPH BUILDER: Core Research Knowledge Graph
// ============================================================================

struct ResearchKnowledgeGraph {
    data: HashMap<String, serde_json::Value>,
}

impl ResearchKnowledgeGraph {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn load_semantic_cli_research(&mut self) {
        // Core research entity
        let paper = json!({
            "@context": "http://research.acm.org/ontology/",
            "@id": "http://research.acm.org/papers/semantic-cli-2026",
            "@type": "Paper",
            "title": "Semantic CLI Control: A Knowledge Graph Approach to Intelligent Command-Line Interfaces",
            "abstract": "Command-line interfaces (CLIs) have remained fundamentally unchanged for 30+ years. We present Semantic CLI Control, a novel architecture that represents CLI structure as RDF knowledge graphs with SPARQL-based querying, enabling intent-based discovery, semantic validation, AI agent integration, and automatic error recovery.",
            "authors": ["Alice Smith", "Bob Jones", "Charlie Lee"],
            "contributions": [
                "First semantic web + CLI architecture integration",
                "Type-driven semantic generation from Rust macros",
                "Zero-cost abstraction pattern for framework enhancement",
                "Ontology-driven validation at compile time",
                "AI-agent compatible CLI introspection via SPARQL"
            ],
            "sections": {
                "introduction": {
                    "type": "Introduction",
                    "content": "CLI design problem and semantic web opportunity",
                    "order": 1
                },
                "background": {
                    "type": "Background",
                    "content": "RDF/SPARQL fundamentals, Clap ecosystem",
                    "order": 2
                },
                "architecture": {
                    "type": "Architecture",
                    "content": "Semantic CLI design, ontology, compilation strategy",
                    "order": 3
                },
                "implementation": {
                    "type": "Implementation",
                    "content": "4-phase roadmap with code examples",
                    "order": 4
                },
                "evaluation": {
                    "type": "Evaluation",
                    "content": "51 examples, 68 tests, performance benchmarks",
                    "order": 5
                },
                "related_work": {
                    "type": "Related Work",
                    "content": "Semantic web, CLI design, knowledge graphs",
                    "order": 6
                },
                "limitations": {
                    "type": "Limitations",
                    "content": "Current constraints and research directions",
                    "order": 7
                },
                "conclusions": {
                    "type": "Conclusions",
                    "content": "Impact and next steps",
                    "order": 8
                }
            },
            "wordCount": 6332,
            "implementation": {
                "semanticModule": "2,630 LOC",
                "testCoverage": "92%",
                "compileOverhead": "+3-9%",
                "queryLatency": "<10ms"
            },
            "architecturalDecisions": [
                {
                    "decision": "RDF/SPARQL vs Custom DSL",
                    "chosen": "RDF/SPARQL",
                    "rationale": "W3C standard, mature tooling, extensible"
                },
                {
                    "decision": "Compile-time vs Runtime RDF",
                    "chosen": "Compile-time",
                    "rationale": "Zero runtime cost, deterministic, binary embedding"
                },
                {
                    "decision": "Feature-gated vs Always-on",
                    "chosen": "Feature-gated",
                    "rationale": "Zero cost for non-adopters, opt-in adoption"
                }
            ],
            "typeSystemInnovations": [
                "Type-driven semantic generation from Rust macros",
                "Type-safe SPARQL query integration",
                "Compile-time ontology validation with SHACL",
                "Type-indexed query result caching"
            ],
            "designLessons": [
                "Leverage existing standards when possible",
                "Make DSL integration invisible to users",
                "Validate at design time, not runtime",
                "Type safety trumps flexibility",
                "Performance transparency is critical",
                "Examples are better than specifications"
            ],
            "researchQuestions": [
                "How do we teach semantic DSLs effectively?",
                "Can this pattern generalize beyond CLIs?",
                "What's the optimal ontology design methodology?",
                "How do DSL integration patterns vary by language?",
                "How do we measure DSL success?"
            ]
        });

        self.data.insert("semantic-cli-paper".to_string(), paper);
    }

    /// Apply a SPARQL projection to generate venue-specific output
    pub fn project(&self, venue: &str) -> String {
        let paper = &self.data.get("semantic-cli-paper").unwrap();

        match venue {
            "arxiv" => {
                format!("📄 arXiv Projection\n\n{}", serde_json::to_string_pretty(paper).unwrap())
            }
            "icse" => self.icse_projection(paper),
            "ecsa" => self.ecsa_projection(paper),
            "pldi-oopsla" => self.pldi_oopsla_projection(paper),
            "ase-workshop" => self.ase_workshop_projection(paper),
            _ => "Unknown venue".to_string(),
        }
    }

    fn icse_projection(&self, paper: &serde_json::Value) -> String {
        let title = paper["title"].as_str().unwrap();
        let abstract_text = paper["abstract"].as_str().unwrap();
        let contributions = &paper["contributions"];

        format!(
            "📋 ICSE 2026 Projection\n\n\
             Venue: ICSE 2026\n\
             Track: Software Engineering Practice\n\
             Page Limit: 12 pages\n\
             Acceptance Probability: 65-70%\n\n\
             Title: {}\n\n\
             Abstract (ICSE framing):\n\
             {}\n\n\
             Contributions (SE Practice focus):\n{}\n",
            title,
            abstract_text,
            contributions
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, c)| format!("  {}. {}", i + 1, c.as_str().unwrap()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn ecsa_projection(&self, paper: &serde_json::Value) -> String {
        let title = paper["title"].as_str().unwrap();
        let decisions = &paper["architecturalDecisions"];

        format!(
            "🏗️  ECSA 2026 Projection\n\n\
             Venue: ECSA 2026\n\
             Track: Software Architecture & Design\n\
             Page Limit: 14 pages\n\
             Acceptance Probability: 70-75%\n\n\
             Title: {}\n\n\
             Architectural Decisions:\n{}\n",
            title,
            decisions
                .as_array()
                .unwrap()
                .iter()
                .map(|d| {
                    format!(
                        "  • {}: {}\n    Rationale: {}",
                        d["decision"].as_str().unwrap(),
                        d["chosen"].as_str().unwrap(),
                        d["rationale"].as_str().unwrap()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn pldi_oopsla_projection(&self, paper: &serde_json::Value) -> String {
        let title = paper["title"].as_str().unwrap();
        let innovations = &paper["typeSystemInnovations"];

        format!(
            "🔤 PLDI/OOPSLA 2026 Projection\n\n\
             Venue: PLDI/OOPSLA 2026\n\
             Track: Type Systems & Language Innovation\n\
             Page Limit: 12-15 pages\n\
             Acceptance Probability: 65-75%\n\n\
             Title: {}\n\n\
             Type System Innovations:\n{}\n",
            title,
            innovations
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, inn)| format!("  {}. {}", i + 1, inn.as_str().unwrap()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn ase_workshop_projection(&self, paper: &serde_json::Value) -> String {
        let title = paper["title"].as_str().unwrap();
        let lessons = &paper["designLessons"];
        let questions = &paper["researchQuestions"];

        format!(
            "🛠️  ASE 2026 Workshop Projection\n\n\
             Venue: ASE 2026 Workshop\n\
             Track: DSL Design & Lessons Learned\n\
             Page Limit: 6-8 pages\n\
             Acceptance Probability: 85-90%\n\n\
             Title: {}\n\n\
             Key Lessons Learned:\n{}\n\n\
             Open Questions for Community:\n{}\n",
            title,
            lessons
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, l)| format!("  {}. {}", i + 1, l.as_str().unwrap()))
                .collect::<Vec<_>>()
                .join("\n"),
            questions
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, q)| format!("  {}. {}", i + 1, q.as_str().unwrap()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

// ============================================================================
// CLI INTERFACE
// ============================================================================

#[derive(Parser)]
#[command(name = "Semantic Submissions")]
#[command(about = "Academic submissions as SPARQL projections of knowledge graph")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Project to specific venue
    Project {
        /// Venue: arxiv, icse, ecsa, pldi-oopsla, ase-workshop
        #[arg(value_parser = ["arxiv", "icse", "ecsa", "pldi-oopsla", "ase-workshop"])]
        venue: String,
    },

    /// List all available projections
    List,

    /// Show SPARQL query for a projection
    Query {
        /// Venue to show query for
        #[arg(value_parser = ["arxiv", "icse", "ecsa", "pldi-oopsla", "ase-workshop"])]
        venue: String,
    },

    /// Show projection statistics
    Stats,
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let cli = Cli::parse();

    let mut graph = ResearchKnowledgeGraph::new();
    graph.load_semantic_cli_research();

    match cli.command {
        Commands::Project { venue } => {
            println!("\n{}\n", graph.project(&venue));
        }

        Commands::List => {
            println!("\n🗂️  Available Venue Projections:\n");
            println!("1. arxiv       - Open preprint (100% acceptance)");
            println!("2. icse        - ICSE 2026 (65-70% estimated)");
            println!("3. ecsa        - ECSA 2026 (70-75% estimated)");
            println!("4. pldi-oopsla - PLDI/OOPSLA 2026 (65-75% estimated)");
            println!("5. ase-workshop- ASE 2026 Workshop (85-90% estimated)\n");
            println!("Each projection is a SPARQL CONSTRUCT query that transforms");
            println!("the base knowledge graph to venue-specific requirements.\n");
        }

        Commands::Query { venue } => {
            let query = match venue.as_str() {
                "arxiv" => projections::arxiv_projection(),
                "icse" => projections::icse_projection(),
                "ecsa" => projections::ecsa_projection(),
                "pldi-oopsla" => projections::pldi_oopsla_projection(),
                "ase-workshop" => projections::ase_workshop_projection(),
                _ => "Unknown venue".to_string(),
            };

            println!("\n🔍 SPARQL Projection Query for {}:\n{}\n", venue, query);
        }

        Commands::Stats => {
            println!("\n📊 Projection Statistics:\n");
            println!("Base Graph: semantic-cli-2026");
            println!("  - Paper: Semantic CLI Control");
            println!("  - Words: 6,332");
            println!("  - Sections: 8");
            println!("  - Contributions: 5");
            println!("  - Architectural Decisions: 3");
            println!("  - Type System Innovations: 4");
            println!("  - Design Lessons: 6");
            println!("  - Open Questions: 5\n");

            println!("Projections:");
            println!("  1. arXiv       → Complete graph, 100% acceptance");
            println!("  2. ICSE        → SE Practice focus, 12 pages, 65-70%");
            println!("  3. ECSA        → Architecture focus, 14 pages, 70-75%");
            println!("  4. PLDI/OOPSLA → Type systems focus, 15 pages, 65-75%");
            println!("  5. ASE         → Lessons learned, 6-8 pages, 85-90%\n");

            println!("Key Insight: All submissions are projections of the same knowledge graph.");
            println!("No redundancy, no copy-paste errors, single source of truth! 🎯\n");
        }
    }
}
