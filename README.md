# Xee

XML Execution Engine.

## What is Xee?

For the time being, Xee consists of an XPath 3.1 implementation. This implementation
is as of yet incomplete.

## XPath architecture

- XPath text is parsed using a Pest grammar in `xee-xpath/src/xpath-31.pest`.

- The parse result is then transformed into an AST (abstract syntax tree),
  defined by `xee-xpath/src/ast.rs`, using `xee-xpath/parse_ast.rs`. The AST is
  easier to manage than the Pest parse result, and already contains some
  desugaring operations.

- The AST is then transformed into an IR (intermediate representation), defined
  by `xee-xpath/src/ir.rs`, using `xee-xpath/ast_ir.rs`. The IR is a lot easier
  to manage than the AST. It's in administrative normal form, meaning the
  result of expressions is assigned to a unique variable (using `let`) before
  use in function calls, binary operations and such.

- The IR is then transformed into bytecode, defined by
  `xee-xpath/src/instruction.rs`, using `xee-xpath/ir_interpret.rs`.

- The interpreter can execute the bytecode. The bytecode interpreter is defined
  by `xee-xpath/src/interpret.rs`. The interpreter is a stack machine.
