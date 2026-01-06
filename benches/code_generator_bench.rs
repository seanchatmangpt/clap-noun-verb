use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
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

#[cfg(feature = "rdf-composition")]
fn code_generation_small(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(10));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let generator = CliCodeGenerator::new().expect("Failed to create generator");

    c.bench_function("codegen_10_verbs", |b| {
        b.iter(|| generator.generate_from_ontology(&parsed))
    });
}

#[cfg(feature = "rdf-composition")]
fn code_generation_medium(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(50));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let generator = CliCodeGenerator::new().expect("Failed to create generator");

    c.bench_function("codegen_50_verbs", |b| {
        b.iter(|| generator.generate_from_ontology(&parsed))
    });
}

#[cfg(feature = "rdf-composition")]
fn code_generation_large(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let generator = CliCodeGenerator::new().expect("Failed to create generator");

    c.bench_function("codegen_100_verbs", |b| {
        b.iter(|| generator.generate_from_ontology(&parsed))
    });
}

#[cfg(feature = "rdf-composition")]
fn code_generation_xl(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(500));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let generator = CliCodeGenerator::new().expect("Failed to create generator");

    c.bench_function("codegen_500_verbs", |b| {
        b.iter(|| generator.generate_from_ontology(&parsed))
    });
}

#[cfg(feature = "rdf-composition")]
fn code_generation_parameterized(c: &mut Criterion) {
    let mut group = c.benchmark_group("code_generation");
    group.sample_size(10); // Smaller sample size for longer operations

    for size in [10, 25, 50, 100, 250].iter() {
        let turtle = black_box(generate_turtle(*size));
        let parser = TurtleParser::new();
        let parsed = parser.parse(&turtle).expect("Failed to parse");
        let generator = CliCodeGenerator::new().expect("Failed to create generator");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_verbs", size)),
            size,
            |b, _| {
                b.iter(|| generator.generate_from_ontology(&parsed))
            },
        );
    }
    group.finish();
}

#[cfg(feature = "rdf-composition")]
fn noun_macro_generation(c: &mut Criterion) {
    let generator = CliCodeGenerator::new().expect("Failed to create generator");

    c.bench_function("noun_macro_generation", |b| {
        b.iter(|| generator.generate_noun_macro(black_box("services"), black_box("Service commands")))
    });
}

#[cfg(feature = "rdf-composition")]
fn verb_macro_generation(c: &mut Criterion) {
    let generator = CliCodeGenerator::new().expect("Failed to create generator");

    c.bench_function("verb_macro_generation", |b| {
        b.iter(|| {
            generator.generate_verb_macro(
                black_box("status"),
                black_box("Services"),
                black_box("status_handler"),
            )
        })
    });
}

#[cfg(feature = "rdf-composition")]
criterion_group!(
    benches,
    code_generation_small,
    code_generation_medium,
    code_generation_large,
    code_generation_xl,
    code_generation_parameterized,
    noun_macro_generation,
    verb_macro_generation
);

#[cfg(not(feature = "rdf-composition"))]
criterion_group!(benches,);

criterion_main!(benches);
