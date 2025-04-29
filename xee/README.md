# xee

Swiss Army knife tool for XML.

The `xee` (try `-h`) allows you to do stuff with XML on the command line.

Features include:

- formatting XML in various ways, including in indented form
- evaluate an XPath expression against an XML document
- a REPL for evaluating XPath expressions
- transform XML documents using XSLT stylesheets.

This implements XPath 3.1 and parts of XSLT 3.0.

## Installation

You can download a pre-build binary of `xee` from the [releases
page](https://github.com/Paligo/xee/releases). You can also [build it
yourself](https://github.com/Paligo/xee/?tab=readme-ov-file#obtaining-the-xee-commandline-tool)
if you have the Rust toolchain installed.

## Usage

### Execute an XPath expression

Execute an XPath expression `/doc/p` against a file `foo.xml`, result to stdout:

```
xee xpath /doc/p foo.xml
```

If you don't include the file, the XML is taken from stdin, allowing you to do:

```
cat foo.xml | xee xpath /doc/p
```

#### Working with namespaces

For XML with namespaces, use the `--namespace` option (format: `prefix=uri`):

```
xee xpath /doc/a:p --namespace a=http://example.com foo.xml
```

For multiple namespaces, repeat the option:

```
xee xpath /x:doc/a:p \
  --namespace x=http://example.com/x \
  --namespace a=http://example.com/a \
  foo.xml
```

For XML with a default namespace, use `--default-namespace-uri`:

```
xee xpath /doc/p --default-namespace-uri http://example.com foo.xml
```

### Interactive shell for XPath

Interactive shell (REPL) to issue multiple xpath expressions against a document:

```
xee repl
```

### Pretty-print an XML file

Pretty-print `foo.xml`, result to stdout:

```
xee indent foo.xml
```

### Transform an XML document with XSLT

Transform an XML document using an XSLT stylesheet:

```
xee xslt stylesheet.xsl input.xml
```

You can also specify an output file:

```
xee xslt stylesheet.xsl input.xml --output result.xml
```

Or read from stdin:

```
cat input.xml | xee xslt stylesheet.xsl
```

## More Xee

This is built using [`xee-xpath`](https://docs.rs/xee-xpath/latest/xee_xpath/),
a high level API to issue XPath 3.1 expressions in Rust.

[Xee homepage](https://github.com/Paligo/xee)

## Credits

This project was made possible by the generous support of
[Paligo](https://paligo.net/).
