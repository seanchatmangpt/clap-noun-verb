use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::turtle_parser::TurtleParser;

/// Generate test Turtle document with N triples
#[cfg(feature = "rdf-composition")]
fn generate_turtle(num_verbs: usize) -> String {
    let mut turtle = String::from(
        "@prefix cnv: <https://cnv.dev/ontology#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n",
    );

    // Create noun
    turtle.push_str("cnv:Services a cnv:Noun ;\n    cnv:name \"services\" ;\n    rdfs:comment \"Service commands\" .\n\n");

    // Create verbs
    for i in 0..num_verbs {
        turtle.push_str(&format!(
            "cnv:Verb{} a cnv:Verb ;\n    cnv:name \"verb{}\" ;\n    cnv:hasNoun cnv:Services ;\n    cnv:description \"Verb {}\";  cnv:handler \"handler{}\" .\n\n",
            i, i, i, i
        ));
    }

    turtle
}

#[cfg(feature = "rdf-composition")]
fn turtle_parsing_small(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(10));
    let parser = TurtleParser::new();

    c.bench_function("turtle_parse_10_verbs", |b| b.iter(|| parser.parse(&turtle)));
}

#[cfg(feature = "rdf-composition")]
fn turtle_parsing_medium(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();

    c.bench_function("turtle_parse_100_verbs", |b| b.iter(|| parser.parse(&turtle)));
}

#[cfg(feature = "rdf-composition")]
fn turtle_parsing_large(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(1000));
    let parser = TurtleParser::new();

    c.bench_function("turtle_parse_1000_verbs", |b| b.iter(|| parser.parse(&turtle)));
}

#[cfg(feature = "rdf-composition")]
fn turtle_parsing_parameterized(c: &mut Criterion) {
    let mut group = c.benchmark_group("turtle_parsing");

    for size in [10, 50, 100, 500, 1000].iter() {
        let turtle = black_box(generate_turtle(*size));
        let parser = TurtleParser::new();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_verbs", size)),
            size,
            |b, _| b.iter(|| parser.parse(&turtle)),
        );
    }
    group.finish();
}

#[cfg(feature = "rdf-composition")]
fn turtle_validation(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");

    c.bench_function("turtle_validate_100_verbs", |b| b.iter(|| parsed.validate_ontology()));
}

#[cfg(feature = "rdf-composition")]
fn prefix_resolution(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");

    c.bench_function("prefix_resolution_100_verbs", |b| b.iter(|| parsed.resolve_prefixes()));
}

#[cfg(feature = "rdf-composition")]
criterion_group!(
    benches,
    turtle_parsing_small,
    turtle_parsing_medium,
    turtle_parsing_large,
    turtle_parsing_parameterized,
    turtle_validation,
    prefix_resolution
);

#[cfg(not(feature = "rdf-composition"))]
criterion_group!(benches,);

criterion_main!(benches);
