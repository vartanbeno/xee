## Environment variables

- fn:available-environment-variables
- fn:environment-variable

Straightforward implementation, except that there should
be a flag to disallow any access (which may be set by default), for untrusted
sources.

## URIs

- fn:base-uri

needs a notion of base URIs on nodes

- fn:document-uri

needs some way to look up the document URI for a node.

## collation

- fn:collection-key

Cannot be supported until icu4x implements it.

## collection

- fn:collection
- fn:uri-collection

A notion of collections in the dynamid context.

## language

- fn:lang

Needs a notion of language on a node.

- fn:default-language

Needs a notion of default language in the static (?) context.

## id support

- fn:element-with-id
- fn:id
- fn:idref

Needs a notion of an id on a node. This can be simply xml:id (once Xot
processes it as an id).

For idref, it needs a notion of an idref type on a node.


## date formatting

- fn:format-date
- fn:format-dateTime
- fn:format-time

Needs enormously complex i18n aware implementation.

## date parsing

- fn:parse-ietf-date

## number formatting

- fn:format-integer
- fn:format-number

Needs enormously complex i18n aware implementation.

## qnames

- fn:in-scope-prefixes

## JSON

- fn:json-doc
- fn:json-to-xml
- fn:parse-json
- fn:xml-to-json

## XML parsing and serializing

- fn:parse-xml
- fn:parse-xml-fragment
- fn:serialize

Some questions about use of base URL

## plaintext

- fn:unparsed-text
- fn:unparsed-text-available
- fn:unparsed-text-lines

Needs a notion of text resources on dynamic context.

## random numbers

- fn:random-number-generator

## schema support

- fn:nilled

## debugging

- fn:trace

## XSLT support

- fn:transform

