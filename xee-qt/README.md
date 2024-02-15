# xee-qt

There is a big xpath conformance suite, and this contains the test runner
for it.

To check against regressions, run:

```
cargo run --release  -- check ../vendor/qt3tests/
```

To run all tests:

```
cargo run --release  -- all ../vendor/qt3tests/
```

You can run the tests and update the regression filter accordingly:

```
cargo run --release -- update ../vendor/qt3tests/
```
