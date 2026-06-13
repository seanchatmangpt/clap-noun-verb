use c8_graph::{Construct8Delta, Construct8Triple, GraphField};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn apply_1_triple(c: &mut Criterion) {
    c.bench_function("apply_1_triple", |b| {
        b.iter(|| {
            let mut delta = Construct8Delta::new();
            let triple = Construct8Triple::new(black_box(1), black_box(2), black_box(3));
            delta.push_checked(triple).unwrap();

            let mut graph = GraphField::new();
            graph.apply_construct8(black_box(&delta)).unwrap();
        });
    });
}

fn apply_2_triples(c: &mut Criterion) {
    c.bench_function("apply_2_triples", |b| {
        b.iter(|| {
            let mut delta = Construct8Delta::new();
            delta
                .push_checked(Construct8Triple::new(black_box(1), black_box(2), black_box(3)))
                .unwrap();
            delta
                .push_checked(Construct8Triple::new(black_box(4), black_box(5), black_box(6)))
                .unwrap();

            let mut graph = GraphField::new();
            graph.apply_construct8(black_box(&delta)).unwrap();
        });
    });
}

fn apply_4_triples(c: &mut Criterion) {
    c.bench_function("apply_4_triples", |b| {
        b.iter(|| {
            let mut delta = Construct8Delta::new();
            for i in 0..4 {
                let s = black_box(i * 3);
                let p = black_box(i * 3 + 1);
                let o = black_box(i * 3 + 2);
                delta.push_checked(Construct8Triple::new(s, p, o)).unwrap();
            }

            let mut graph = GraphField::new();
            graph.apply_construct8(black_box(&delta)).unwrap();
        });
    });
}

fn apply_8_triples(c: &mut Criterion) {
    c.bench_function("apply_8_triples", |b| {
        b.iter(|| {
            let mut delta = Construct8Delta::new();
            for i in 0..8 {
                let s = black_box(i as u64 * 3);
                let p = black_box(i as u64 * 3 + 1);
                let o = black_box(i as u64 * 3 + 2);
                delta.push_checked(Construct8Triple::new(s, p, o)).unwrap();
            }

            let mut graph = GraphField::new();
            graph.apply_construct8(black_box(&delta)).unwrap();
        });
    });
}

criterion_group!(benches, apply_1_triple, apply_2_triples, apply_4_triples, apply_8_triples);
criterion_main!(benches);
