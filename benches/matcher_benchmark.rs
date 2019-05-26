extern crate criterion;

use criterion::*;

extern crate market_matcher;

use market_matcher::*;

fn l_benchmark(c: &mut Criterion) {
    let mut book = OrderBook::default();
    for i in 0..7000 {
        let request = Request {
            price: 1,
            size: 1,
            side: Side::Sell,
            request_type: Type::Limit,
            user_id: i
        };
        book.match_request(&request.clone());
    }
    let request = Request {
        price: 1,
        size: 20,
        side: Side::Buy,
        request_type: Type::Limit,
        user_id: 10000
    };
    c.bench_function("Limit matching", move |b| {
        b.iter_batched_ref(
            || (book.clone(), request.clone()),
            |(book, request)| book.match_request(black_box(request)),
            BatchSize::SmallInput
            );
    });
}

fn ic_benchmark(c: &mut Criterion) {
    let mut book = OrderBook::default();
    for i in 0..7000 {
        let request = Request {
            price: 1,
            size: 1,
            side: Side::Sell,
            request_type: Type::Limit,
            user_id: i
        };
        book.match_request(&request.clone());
    }
    let request = Request {
        price: 1,
        size: 20,
        side: Side::Buy,
        request_type: Type::ImmediateOrCancel,
        user_id: 10000
    };
    c.bench_function("ImmediateOrCancel matching", move |b| {
        b.iter_batched_ref(
            || (book.clone(), request.clone()),
            |(book, request)| book.match_request(black_box(request)),
            BatchSize::SmallInput
        );
    });
}

fn fk_benchmark(c: &mut Criterion) {
    let mut book = OrderBook::default();
    for i in 0..7000 {
        let request = Request {
            price: 1,
            size: 1,
            side: Side::Sell,
            request_type: Type::Limit,
            user_id: i
        };
        book.match_request(&request.clone());
    }
    let request = Request {
        price: 1,
        size: 20,
        side: Side::Buy,
        request_type: Type::FillOrKill,
        user_id: 10000
    };
    c.bench_function("FillOrKill matching", move |b| {
        b.iter_batched_ref(
            || (book.clone(), request.clone()),
            |(book, request)| book.match_request(request),
            BatchSize::SmallInput
        );
    });
}

criterion_group!(benches, l_benchmark, ic_benchmark, fk_benchmark);
// criterion_group!(benches, fk_benchmark);
criterion_main!(benches);
