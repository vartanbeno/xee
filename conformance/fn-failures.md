# fn test failures that we cannot explain due to lack of implementation

## fn/data.xml 

### K2-DataFunc-6

I think this test failure has something to do with the lack of schema support.

## fn:distinct-values

### fn-distinct-values-mixed-args-005

I'm not sure whether this test failure is correct. We do get a 0 and a 1,
just that the zero is a float. Needs more analysis to figure out whether
the test isn't accurate or whether the implementation has problems.

## fn:doc

### fn-doc-37

Could be due to missing implementation of `id` function and any of not giving
good feedback.

### K2-SeqDocFunc-3

This fails with a XPTY0004. Why? Is it because untypedAtomic is not allowed as an argument?

