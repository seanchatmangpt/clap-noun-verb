use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::sparql_executor_oxigraph::SparqlExecutor;
#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::code_generator::CliCodeGenerator;

/// Generate test Turtle document with N verbs
#[cfg(feature = "rdf-composition")]
fn generate_turtle(num_verbs: usize) -> String {
    let mut turtle = String::from(
        "@prefix cnv: <https://cnv.dev/ontology#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n"
    );

    turtle.push_str("cnv:Services a cnv:Noun ;\n    cnv:name \"services\" ;\n    rdfs:comment \"Service commands\" .\n\n");

    for i in 0..num_verbs {
        turtle.push_str(&format!(
            "cnv:Verb{} a cnv:Verb ;\n    cnv:name \"verb{}\" ;\n    cnv:hasNoun cnv:Services ;\n    cnv:description \"Verb {}\";  cnv:handler \"handler{}\" .\n\n",
            i, i, i, i
        ));
    }

    turtle
}

/// Full workflow: Parse → Validate → Generate
#[cfg(feature = "rdf-composition")]
fn end_to_end_small(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(10));

    c.bench_function("e2e_parse_validate_generate_10_verbs", |b| {
        b.iter(|| {
            let parser = TurtleParser::new();
            let parsed = parser.parse(&turtle).expect("Parse failed");
            let generator = CliCodeGenerator::new().expect("Generator creation failed");
            generator.generate_from_ontology(&parsed).expect("Generation failed")
        })
    });
}

#[cfg(feature = "rdf-composition")]
fn end_to_end_medium(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(50));

    c.bench_function("e2e_parse_validate_generate_50_verbs", |b| {
        b.iter(|| {
            let parser = TurtleParser::new();
            let parsed = parser.parse(&turtle).expect("Parse failed");
            let generator = CliCodeGenerator::new().expect("Generator creation failed");
            generator.generate_from_ontology(&parsed).expect("Generation failed")
        })
    });
}

#[cfg(feature = "rdf-composition")]
fn end_to_end_large(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));

    c.bench_function("e2e_parse_validate_generate_100_verbs", |b| {
        b.iter(|| {
            let parser = TurtleParser::new();
            let parsed = parser.parse(&turtle).expect("Parse failed");
            let generator = CliCodeGenerator::new().expect("Generator creation failed");
            generator.generate_from_ontology(&parsed).expect("Generation failed")
        })
    });
}

/// Full workflow with SPARQL discovery
#[cfg(feature = "rdf-composition")]
fn end_to_end_with_discovery_medium(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(50));

    c.bench_function("e2e_with_sparql_discovery_50_verbs", |b| {
        b.iter(|| {
            let parser = TurtleParser::new();
            let parsed = parser.parse(&turtle).expect("Parse failed");

            let executor = SparqlExecutor::new(&parsed).expect("Executor creation failed");
            let _classes = executor.list_classes().expect("List classes failed");
            let _properties = executor.list_properties().expect("List properties failed");

            let generator = CliCodeGenerator::new().expect("Generator creation failed");
            generator.generate_from_ontology(&parsed).expect("Generation failed")
        })
    });
}

#[cfg(feature = "rdf-composition")]
fn end_to_end_with_discovery_large(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));

    c.bench_function("e2e_with_sparql_discovery_100_verbs", |b| {
        b.iter(|| {
            let parser = TurtleParser::new();
            let parsed = parser.parse(&turtle).expect("Parse failed");

            let executor = SparqlExecutor::new(&parsed).expect("Executor creation failed");
            let _classes = executor.list_classes().expect("List classes failed");
            let _properties = executor.list_properties().expect("List properties failed");

            let generator = CliCodeGenerator::new().expect("Generator creation failed");
            generator.generate_from_ontology(&parsed).expect("Generation failed")
        })
    });
}

/// Parameterized end-to-end workflow
#[cfg(feature = "rdf-composition")]
fn end_to_end_parameterized(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_workflow");
    group.sample_size(10); // Smaller sample for longer operations

    for size in [10, 25, 50, 100].iter() {
        let turtle = black_box(generate_turtle(*size));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("complete_{}_verbs", size)),
            size,
            |b, _| {
                b.iter(|| {
                    let parser = TurtleParser::new();
                    let parsed = parser.parse(&turtle).expect("Parse failed");
                    let generator = CliCodeGenerator::new().expect("Generator creation failed");
                    generator.generate_from_ontology(&parsed).expect("Generation failed")
                })
            },
        );
    }
    group.finish();
}

/// Measure overhead of parser creation
#[cfg(feature = "rdf-composition")]
fn parser_creation_overhead(c: &mut Criterion) {
    c.bench_function("turtle_parser_creation", |b| {
        b.iter(|| TurtleParser::new())
    });
}

/// Measure overhead of generator creation
#[cfg(feature = "rdf-composition")]
fn generator_creation_overhead(c: &mut Criterion) {
    c.bench_function("code_generator_creation", |b| {
        b.iter(|| CliCodeGenerator::new().expect("Generator creation failed"))
    });
}

/// Measure overhead of executor creation
#[cfg(feature = "rdf-composition")]
fn executor_creation_overhead(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(50));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Parse failed");

    c.bench_function("sparql_executor_creation_50_verbs", |b| {
        b.iter(|| SparqlExecutor::new(black_box(&parsed)).expect("Executor creation failed"))
    });
}

#[cfg(feature = "rdf-composition")]
criterion_group!(
    benches,
    end_to_end_small,
    end_to_end_medium,
    end_to_end_large,
    end_to_end_with_discovery_medium,
    end_to_end_with_discovery_large,
    end_to_end_parameterized,
    parser_creation_overhead,
    generator_creation_overhead,
    executor_creation_overhead
);

#[cfg(not(feature = "rdf-composition"))]
criterion_group!(benches,);

criterion_main!(benches);
