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

## fn:lower-case

### fn-lower-case-19

There's a difference between the value 1011 (greek yot) that we get and a 895
(greek and coptic) that it expects. Everything else is correct. I expect this
is due to unicode table differences, but who knows.

Here's some debugging code that can be installed in assert permutation in
the test runner to help debug this:

```
    for (a, b) in sequence
        .items()
        .unwrap()
        .zip(expected_sequence.items().unwrap())
    {
        if a != b {
            println!("WHOAH a: {:?} b: {:?}", a, b);
        } else {
            println!("a: {:?} b: {:?}", a, b);
        }
    }
```

