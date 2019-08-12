# Система сведения заявок для рынка – матчер.

`cargo test` - тесты, что находятся в `src/tests.rs`  
`cargo run` - запуск, использует заявки в файле requests.json  
`cargo bench` - запуск бенчмарков в `src/benches/matcher_benchmark.rs`

Результаты бенчмарков для матчинга входящей заявки, которая сводится с 20 из очереди в 7000 (`на Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz`) (`RUSTFLAGS="-C target-cpu=native" cargo bench`):

```
                                Lower bound   Estimate  Upper bound
Limit matching          time:   [296.08 ns 297.44 ns 299.15 ns]
Limit quiet matching    time:   [97.325 ns 98.452 ns 99.405 ns]
```

Изначальная имплементация имела скорость выполнения 90 us.

