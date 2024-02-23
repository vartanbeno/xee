# xee-qt

There is a big [xpath conformance suite](https://github.com/w3c/qt3tests), of
which we've added a copy under `vendor`, and this contains the test runner for
it.

You can install the test runner, but that makes it difficult to check
immediately against changes in the codebase. The instructions here describe how
to run the tests against the current state of the Xee project. We compile in
`--release` mode as otherwise running the tests takes a long time.

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
