use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::sparql_executor_oxigraph::SparqlExecutor;
#[cfg(feature = "rdf-composition")]
use clap_noun_verb::rdf::turtle_parser::TurtleParser;

/// Generate test Turtle document with N verbs
#[cfg(feature = "rdf-composition")]
fn generate_turtle(num_verbs: usize) -> String {
    let mut turtle = String::from(
        "@prefix cnv: <https://cnv.dev/ontology#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n",
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
fn sparql_list_classes_small(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(10));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    c.bench_function("sparql_list_classes_10_verbs", |b| b.iter(|| executor.list_classes()));
}

#[cfg(feature = "rdf-composition")]
fn sparql_list_classes_medium(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    c.bench_function("sparql_list_classes_100_verbs", |b| b.iter(|| executor.list_classes()));
}

#[cfg(feature = "rdf-composition")]
fn sparql_list_classes_large(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(1000));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    c.bench_function("sparql_list_classes_1000_verbs", |b| b.iter(|| executor.list_classes()));
}

#[cfg(feature = "rdf-composition")]
fn sparql_list_properties(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    c.bench_function("sparql_list_properties_100_verbs", |b| b.iter(|| executor.list_properties()));
}

#[cfg(feature = "rdf-composition")]
fn sparql_simple_select_query(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = black_box(
        "PREFIX cnv: <https://cnv.dev/ontology#> \
         SELECT ?v WHERE { ?v a cnv:Verb }",
    );

    c.bench_function("sparql_select_all_verbs_100", |b| b.iter(|| executor.execute_query(query)));
}

#[cfg(feature = "rdf-composition")]
fn sparql_filter_query(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = black_box(
        "PREFIX cnv: <https://cnv.dev/ontology#> \
         SELECT ?v WHERE { \
           ?v a cnv:Verb ; \
           cnv:name ?name . \
           FILTER(regex(?name, \"verb[0-9]+\")) \
         }",
    );

    c.bench_function("sparql_filter_query_100_verbs", |b| b.iter(|| executor.execute_query(query)));
}

#[cfg(feature = "rdf-composition")]
fn sparql_join_query(c: &mut Criterion) {
    let turtle = black_box(generate_turtle(100));
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle).expect("Failed to parse");
    let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

    let query = black_box(
        "PREFIX cnv: <https://cnv.dev/ontology#> \
         SELECT ?v ?n WHERE { \
           ?v a cnv:Verb ; \
           cnv:hasNoun ?n . \
           ?n a cnv:Noun \
         }",
    );

    c.bench_function("sparql_join_query_100_verbs", |b| b.iter(|| executor.execute_query(query)));
}

#[cfg(feature = "rdf-composition")]
fn sparql_parameterized_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparql_queries");

    for size in [10, 50, 100, 500, 1000].iter() {
        let turtle = black_box(generate_turtle(*size));
        let parser = TurtleParser::new();
        let parsed = parser.parse(&turtle).expect("Failed to parse");
        let executor = SparqlExecutor::new(&parsed).expect("Failed to create executor");

        let query = "PREFIX cnv: <https://cnv.dev/ontology#> SELECT ?v WHERE { ?v a cnv:Verb }";

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("select_{}_verbs", size)),
            size,
            |b, _| b.iter(|| executor.execute_query(query)),
        );
    }
    group.finish();
}

#[cfg(feature = "rdf-composition")]
criterion_group!(
    benches,
    sparql_list_classes_small,
    sparql_list_classes_medium,
    sparql_list_classes_large,
    sparql_list_properties,
    sparql_simple_select_query,
    sparql_filter_query,
    sparql_join_query,
    sparql_parameterized_queries
);

#[cfg(not(feature = "rdf-composition"))]
criterion_group!(benches,);

criterion_main!(benches);
