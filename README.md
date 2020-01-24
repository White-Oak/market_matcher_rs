# Система сведения заявок для рынка – матчер.

`cargo test` - тесты, что находятся в `src/tests.rs`  
`cargo run` - запуск, использует заявки в файле requests.json  
`cargo bench` - запуск бенчмарков в `src/benches/matcher_benchmark.rs`

Результаты бенчмарков для матчинга входящей заявки, которая сводится с 20 из очереди в 7000  (`RUSTFLAGS="-C target-cpu=native" cargo bench`):

### на `Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz`

```
                                Lower bound   Estimate  Upper bound
Limit matching          time:   [296.08 ns 297.44 ns 299.15 ns]
Limit quiet matching    time:   [97.325 ns 98.452 ns 99.405 ns]
```

Изначальная имплементация имела скорость выполнения 90 us, то есть примерно в тысячу раз медленнее.

### на `Intel(R) Core(TM) i7-8650U CPU @ 1.90GHz`

```
                                Lower bound   Estimate  Upper bound
Limit matching          time:   [282.12 ns 283.60 ns 285.78 ns]
Limit quiet matching    time:   [105.92 ns 107.39 ns 108.58 ns]
```
