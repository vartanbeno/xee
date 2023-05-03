# Implementation Notes

I'm grateful to all the authors of the various libraries I've used to implement
this code. Here are various non-library resources I've relied on during the
implementation.

The book [Crafting Interpreters](https://craftinginterpreters.com/) by Robert
Nystrom was very helpful in designing the XPath interpreter, in particular the
operation around variables and function calls.

[loxcraft](https://github.com/ajeetdsouza/loxcraft) is a Rust implementation of
Lox, the language introduced by Crafting Interpreters. I studied its source
code with interest and picked up a few tricks.

The intermediate representation design is inspired by various lecture notes:

https://course.ccs.neu.edu/cs4410/lec_anf_notes.html

https://maxsnew.com/teaching/eecs-483-fa21/lec_anf_notes.html

https://users.dcc.uchile.cl/~etanter/CC5116/lec_let-and-stack_notes.html
