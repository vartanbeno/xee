# xee-testrunner

This is a test runner that can run both XPath and XSLT conformance test suits.

We have added both the [XPath conformance test
suite](https://github.com/w3c/qt3tests) and the [XSLT conformance test
suite](git@github.com:w3c/xslt30-test.git) under the `vendor` directory of this
project, as `vendor/xpath-tests` and `vendor/xslt-tests`.

The test runner will detect automatically whether you're running the XPath or
XSLT tests, and adjust its behavior accordingly.

You can install this test runner using `cargo install`, but that makes it
difficult to check immediately against changes in the Xee codebase itself,
which is what is it for. The instructions here describe how to run the tests
against the current state of the Xee project. We compile in `--release` mode as
otherwise running the tests takes a long time.

To check against regressions, run:

```
cargo run --release  -- check ../vendor/xpath-tests/
```

or

```
cargo run --release  -- check ../vendor/xslt-tests/
```

To run all tests (for XPath or XSLT):

```
cargo run --release  -- all ../vendor/xpath-tests/
cargo run --release  -- all ../vendor/xslt-tests/
```

You can run the tests and update the regression filter accordingly:

```
cargo run --release -- update ../vendor/xpath-tests/
```
