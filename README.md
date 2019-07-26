# Система сведения заявок для рынка – матчер.

`cargo test` - тесты, что находятся в `src/tests.rs`  
`cargo run` - запуск, использует заявки в файле requests.json  
`cargo bench` - запуск бенчмарков в `src/benches/matcher_benchmark.rs`

Результаты бенчмарков для матчинга входящей заявки, которая сводится с 20 из очереди в 7000 (`на Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz`):

```
                                Lower bound   Estimate  Upper bound
Limit matching          time:   [330.75 ns 336.21 ns 342.61 ns]
Limit quiet matching    time:   [135.04 ns 138.24 ns 140.96 ns]
```

Изначальная имплементация имела скорость выполнения 90 us.

