# Wołacz dla polskich imion

Funkcja implementuje odmiane polskich imion w wołaczu. Może być uzyta w programie, aby np witać użytkownika,
którego imię jest znane w mianowniku.
    
Jesli program ma bazę danych uzytkownikow, jako parametr wejsciowy
podajemy imię, a dostajemy odmienione słowo w wołaczu.

```
> cargo test

    Finished test [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/debug/deps/wolacz-6f34bc4b377b841c)

running 2 tests
test tests::imiona_meskie ... ok
test tests::imiona_zenskie ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
