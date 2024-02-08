## Unit Test (watch)

```sh
cargo watch -q -c -x "test -- --nocapture"

# Specific test with filter.
cargo watch -q -c -x "test -p lib-core test_create -- --nocapture"
```

## Unit Test

```sh
cargo test -- --nocapture