# xee-interpreter

`xee-interpreter` provides the runtime to run Xee code. It includes a
bytecode interpreter, an implementation of XPath data types, and the XPath
standard library.

It is a low-level component, and is used by `xee-xpath` to implement XPath.

## How to rebuild the .postcard file

To rebuild the ICU postcard file you need icu4x-datagen installed. Then, go to the top-level directory (not the `xee-xpath` subdirectory), do a `cargo build` (so the `xee` binary) is built.

Then we can run this command:

```bash
icu4x-datagen --format blob --keys-for-bin ./target/debug/xee --locales full --cldr-tag latest --out xee-xpath/buffer_data.postcard
```

The keys are automatically extracted from the binary, so that it only downloadsq ICU information required for the XPath engine, nothing more.

Eventually we'd like to automate this in a build script, but this will do for now.
