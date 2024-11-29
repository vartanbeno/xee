use divan::{black_box, Bencher};

use xee_xpath::{Documents, Queries, Query};

fn main() {
    divan::main();
}

#[divan::bench]
fn range(bencher: Bencher) {
    let queries = Queries::default();
    let mut q = queries.sequence("1 to 1000").unwrap();

    let mut documents = Documents::new();

    bencher.bench_local(move || {
        black_box(&mut q)
            .execute_build_context(&mut documents, |_build| ())
            .unwrap();
    });
}

#[divan::bench]
fn string_concat(bencher: Bencher) {
    let queries = Queries::default();
    let mut q = queries.sequence("concat('foo', 'bar')").unwrap();

    let mut documents = Documents::new();

    bencher.bench_local(move || {
        black_box(&mut q)
            .execute_build_context(&mut documents, |_build| ())
            .unwrap();
    });
}

#[divan::bench]
fn string_value(bencher: Bencher) {
    let mut documents = Documents::new();
    let handle = documents
        .add_string_without_uri("<doc>Hello world</doc>")
        .unwrap();

    let queries = Queries::default();
    let mut q = queries.sequence("/doc/string()").unwrap();

    bencher.bench_local(move || {
        black_box(&mut q).execute(&mut documents, handle).unwrap();
    });
}
