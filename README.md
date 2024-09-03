# Xee

XML Execution Engine.

## What is Xee?

Xee implements the following:

- A reasonably complete XPath 3.1 implementation.

- An early XSLT 3.0 implementation.

Xee implements these as a bytecode interpreter, in Rust. The XPath functions
are implemented in Rust using a Rust binding system.

This project undergoes extensive automated testing using using specific
developer tests written with `cargo test` as well as using the `xee-testrunner`
infrastructure for running conformance tests.

An affiliated project is `regexml`, which contains an XML Schema and XPath
compatible version of regular expressions for Rust.

## What's missing?

### XPath

- While much of it is covered, parts of the functions in [XPath and XQuery
  Functions and Operators 3.1](https://www.w3.org/TR/xpath-functions-31/) are
  not yet implemented. Contributions are welcome!

- Of the 21859 tests in the QT3 test suite (vendored into `vendor/xpath-tests`)
  that match the features we support (so excluding Query tests), we have 19060
  passing tests. The failures are mostly due to missing library implementation.

- XMLSchema support. While the basic `xs:*` data types as defined by XML Schema
  are implemented, deep XML Schema integration does not exist.

- The Rust binding system for XPath can only be used to implement standard
  library functions - support for extension functions needs to be created.

- No significant optimization work has been done. We do believe Xee provides a
  solid basis for optimization work.

### XSLT

- XSLT support is early days. The basic infrastructure of compiling XSLT
  to bytecode has been implemented, including a full XSLT AST, but much XSLT
  functionality yet remains to be implemented.

## Architecture

XPath gets lexed into tokens using a lexer. This is then turned into an XPath
AST (abstract syntax tree). This AST is then compiled down into a specialized
IR (intermediate representation) which normalizes all variables and simplifies
the code a lot. This IR is then compiled down into a bytecode, executed using a
specialized interpreter.

XPath library functions are implemented with a special Rust binding system based
around Rust macros, which allows you to create Rust functions and register them
into XPath.

XSLT support is very similar: XSLT XML is parsed, then turned into an XSLT AST.
Any embedded XPath expressions are also transformed into the XPath AST. XSLT is
then compiled into the IR, and the IR is compiled into bytecode using the same
infrastructure as for XPath.

## Project structure

### Crates

The Xee project is composed of many crates. Here is a quick overview:

- `xee` - the start of a CLI tool for using Xee.

- `xee-interpreter` - the core virtual machine interpreter that can execute XPath and
  XSLT. Also contains the functions and operators implementation.

- `xee-ir` - an intermediate language (in functional single assignment form)
  with logic to compile it down to Xee bytecode used by `xee-interpreter`.

- `xee-name` - support code for XML namespaces

- `xee-schema-type` - support code defininig properties of core XML schema
  basic datatypes (`xs:*`).

- `xee-testrunner` - a testrunner that can run the QT3 conformance suite of
  XPath tests. It has also been generalized towards supporting running XSLT
  conformance tests, but that implementation is not complete yet.

- `xee-xpath` - combines the underlying components to provide a high level API
  to support XPath queries. Compiles XPath AST provided by `xee-xpath-ast` to
  IR supported by `xee-ir`, which it then uses to create bytecode for
  `xee-interpreter`.

- `xee-xpath-ast` - Defines an XPath AST. Turns `xee-xpath-lexer` output into
  an XPath AST.

- `xee-xpath-lexer` - A lexer for XPath expressions.

- `xee-xpath-load` - Infastructure to help defining loaders for XML data used
  by `xee-testrunner`.

- `xee-xpath-macros` - Macros used by `xee-interpreter` to help implement the
  XPath library functions. Provides a way to create Rust bindings for XPath,
  though is currently restricted to the functions and operator specification.

- `xee-xpath-type` - The AST specifically for type expressions in the XPath
  AST. These are used separately by `xee-xpath-macros` for its Rust bindings
  infrastructure.

- `xee-xslt` - The start of compiler of the XSLT AST (defined by
  `xee-xslt-ast`) into `xee-ir` IR, so that XSLT code can be run by the
  `xee-interpreter` engine.

- `xee-xslt-ast` - Parse XSLT documents into an AST. Uses `xee-xpath-ast` for
  the underlying XPath expressions.

### Other directories

- `conformance` - a manual tracking of various features of XPath and XSLT

- `vendor` - the QT3 test suite (`xpath-tests`) and `xslt-tests` vendored into
  this project for purposes of easy of access and stability.
