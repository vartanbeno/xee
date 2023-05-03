use thiserror::Error;

// based on https://www.w3.org/TR/xpath-functions-31/#error-summary

#[derive(Debug, Error)]
pub enum Error {
    /// Wrong number of arguments.
    ///
    /// Raised when fn:apply is called and the arity
    /// of the supplied function is not the same as the number of members in
    /// the supplied array.
    #[error("Wrong number of arguments")]
    FOAP0001,
    /// Division by zero.
    ///
    /// This error is raised whenever an attempt is made to divide by zero.
    #[error("Division by zero")]
    FOAR0001,
    /// Numeric operation overflow/underflow.
    ///
    /// This error is raised whenever numeric operations result in an overflow or underflow.
    #[error("Numeric operation overflow/underflow")]
    FOAR0002,
    /// Array index out of bounds.
    ///
    /// This error is raised when an integer used to select a member of an array is outside the range of values for that array.
    #[error("Array index out of bounds")]
    FOAY0001,
    /// Negative array length.
    ///
    /// This error is raised when the $length argument to array:subarray is negative.
    #[error("Negative array length")]
    FOAY0002,
    /// Input value too large for decimal.
    ///
    /// Raised when casting to xs:decimal if the supplied value exceeds the implementation-defined limits for the datatype.
    #[error("Input value too large for decimal")]
    FOCA0001,
    /// Invalid lexical value.
    ///
    /// Raised by fn:resolve-QName and fn:QName when a supplied value does not
    /// have the lexical form of a QName or URI respectively; and when casting
    /// to decimal, if the supplied value is NaN or Infinity.
    #[error("Invalid lexical value")]
    FOCA0002,
    /// Input too large for integer.
    ///
    /// Raised when casting to xs:integer if the supplied value exceeds the implementation-defined limits for the datatype.
    #[error("Input too large for integer")]
    FOCA0003,
    /// NaN supplied as float/double value.
    ///
    /// Raised when multiplying or dividing a duration by a number, if the
    /// number supplied is NaN.
    #[error("NaN supplied as float/double value")]
    FOCA0005,
    /// String to be cast to decimal has too many digits of precision.
    ///
    /// Raised when casting a string to xs:decimal if the string has more
    /// digits of precision than the implementation can represent (the
    /// implementation also has the option of rounding).
    #[error("String to be cast to decimal has too many digits of precision")]
    FOCA0006,
    /// Codepoint not valid.
    ///
    /// Raised by fn:codepoints-to-string if the input contains an integer that is not the codepoint of a valid XML character.
    #[error("Codepoint not valid")]
    FOCH0001,
    /// Unsupported collation.
    ///
    /// Raised by any function that uses a collation if the requested collation
    /// is not recognized.
    #[error("Unsupported collation")]
    FOCH0002,
    /// Unsupported normalization form.
    ///
    /// Raised by fn:normalize-unicode if the requested normalization form is
    /// not supported by the implementation.
    #[error("Unsupported normalization form")]
    FOCH0003,
    /// Collation does not support collation units.
    ///
    /// Raised by functions such as fn:contains if the requested collation does
    /// not operate on a character-by-character basis.
    #[error("Collation does not support collation units")]
    FOCH0004,
    /// No context document.
    ///
    /// Raised by fn:id, fn:idref, and fn:element-with-id if the node that
    /// identifies the tree to be searched is a node in a tree whose root is
    /// not a document node.
    #[error("No context document")]
    FODC0001,
    /// Error retrieving resource.
    ///
    /// Raised by fn:doc, fn:collection, and fn:uri-collection to indicate that
    /// either the supplied URI cannot be dereferenced to obtain a resource, or
    /// the resource that is returned is not parseable as XML.
    #[error("Error retrieving resource")]
    FODC0002,
    /// Function not defined as deterministic.
    ///
    /// Raised by fn:doc, fn:collection, and fn:uri-collection to indicate that
    /// it is not possible to return a result that is guaranteed deterministic.
    #[error("Function not defined as deterministic")]
    FODC0003,
    /// Invalid collection URI.
    ///
    /// Raised by fn:collection and fn:uri-collection if the argument is not
    /// a valid xs:anyURI.
    #[error("Invalid collection URI")]
    FODC0004,
    /// Invalid argument to fn:doc or fn:doc-available.
    ///
    /// Raised (optionally) by fn:doc and fn:doc-available if the argument is
    /// not a valid URI reference.
    #[error("Invalid argument to fn:doc or fn:doc-available")]
    FODC0005,
    /// String passed to fn:parse-xml is not a well-formed XML document.
    ///
    /// Raised by fn:parse-xml if the supplied string is not a well-formed and
    /// namespace-well-formed XML document; or if DTD validation is requested
    /// and the document is not valid against its DTD.
    #[error("String passed to fn:parse-xml is not a well-formed XML document")]
    FODC0006,
    /// The processor does not support serialization.
    ///
    /// Raised when fn:serialize is called and the processor does not support
    /// serialization, in cases where the host language makes serialization an
    /// optional feature.
    #[error("The processor does not support serialization")]
    FODC0010,
    /// Invalid decimal format name.
    ///
    /// This error is raised if the decimal format name supplied to
    /// fn:format-number is not a valid QName, or if the prefix in the QName is
    /// undeclared, or if there is no decimal format in the static context with
    /// a matching name.
    #[error("Invalid decimal format name")]
    FODF1280,
    /// Invalid decimal format picture string.
    ///
    /// This error is raised if the picture string supplied to fn:format-number
    /// or fn:format-integer has invalid syntax.
    #[error("Invalid decimal format picture string")]
    FODF1310,
    /// Overflow/underflow in date/time operation.
    ///
    /// Raised when casting to date/time datatypes, or performing arithmetic
    /// with date/time values, if arithmetic overflow or underflow occurs.
    #[error("Overflow/underflow in date/time operation")]
    FODT0001,
    /// err:FODT0002, Overflow/underflow in duration operation.
    ///
    /// Raised when casting to duration datatypes, or performing arithmetic
    /// with duration values, if arithmetic overflow or underflow occurs.
    #[error("Overflow/underflow in duration operation")]
    FODT0002,
    /// Invalid timezone value.
    ///
    /// Raised by adjust-date-to-timezone and related functions if the supplied
    /// timezone is invalid.
    #[error("Invalid timezone value")]
    FODT0003,
    /// Unidentified error.
    ///
    /// Error code used by fn:error when no other error code is provided.
    #[error("Unidentified error")]
    FOER0000,
    /// Invalid date/time formatting parameters.
    ///
    /// This error is raised if the picture string or calendar supplied to
    /// fn:format-date, fn:format-time, or fn:format-dateTime has invalid
    /// syntax.
    #[error("Invalid date/time formatting parameters")]
    FOFD1340,
    /// Invalid date/time formatting component.
    ///
    /// This error is raised if the picture string supplied to fn:format-date
    /// selects a component that is not present in a date, or if the picture
    /// string supplied to fn:format-time selects a component that is not
    /// present in a time.
    #[error("Invalid date/time formatting component")]
    FOFD1350,
    /// JSON syntax error.
    ///
    /// Raised by functions such as fn:json-doc, fn:parse-json or
    /// fn:json-to-xml if the string supplied as input does not conform to the
    /// JSON grammar (optionally with implementation-defined extensions).
    #[error("JSON syntax error")]
    FOJS0001,
    /// JSON duplicate keys.
    ///
    /// Raised by functions such as map:merge, fn:json-doc, fn:parse-json or
    /// fn:json-to-xml if the input contains duplicate keys, when the chosen
    /// policy is to reject duplicates.
    #[error("JSON duplicate keys")]
    FOJS0003,
    /// JSON: not schema-aware.
    ///
    /// Raised by fn:json-to-xml if validation is requested when the processor
    /// does not support schema validation or typed nodes.
    #[error("JSON: not schema-aware")]
    FOJS0004,
    /// Invalid options.
    ///
    /// Raised by functions such as map:merge, fn:parse-json, and
    /// fn:xml-to-json if the $options map contains an invalid entry.
    #[error("Invalid options")]
    FOJS0005,
    /// Invalid XML representation of JSON.
    ///
    /// Raised by fn:xml-to-json if the XML input does not conform to the rules
    /// for the XML representation of JSON.
    #[error("Invalid XML representation of JSON")]
    FOJS0006,
    /// Bad JSON escape sequence.
    ///
    /// Raised by fn:xml-to-json if the XML input uses the attribute
    /// escaped="true" or escaped-key="true", and the corresponding string or
    /// key contains an invalid JSON escape sequence.
    #[error("Bad JSON escape sequence")]
    FOJS0007,
    /// No namespace found for prefix.
    ///
    /// Raised by fn:resolve-QName and analogous functions if a supplied QName
    /// has a prefix that has no binding to a namespace.
    #[error("No namespace found for prefix")]
    FONS0004,
    /// Base-uri not defined in the static context.
    ///
    /// Raised by fn:resolve-uri if no base URI is available for resolving a
    /// relative URI.
    #[error("Base-uri not defined in the static context")]
    FONS0005,
    /// Module URI is a zero-length string.
    ///
    /// Raised by fn:load-xquery-module if the supplied module URI is zero-length.
    #[error("Module URI is a zero-length string")]
    FOQM0001,
    /// Module URI not found.
    ///
    /// Raised by fn:load-xquery-module if no module can be found with the
    /// supplied module URI.
    #[error("Module URI not found")]
    FOQM0002,
    /// Static error in dynamically-loaded XQuery module.
    ///
    /// Raised by fn:load-xquery-module if a static error (including a
    /// statically-detected type error) is encountered when processing the
    /// library module.
    #[error("Static error in dynamically-loaded XQuery module")]
    FOQM0003,
    /// Parameter for dynamically-loaded XQuery module has incorrect type.
    ///
    /// Raised by fn:load-xquery-module if a value is supplied for the initial
    /// context item or for an external variable, and the value does not
    /// conform to the required type declared in the dynamically loaded module.
    #[error("Parameter for dynamically-loaded XQuery module has incorrect type")]
    FOQM0005,
    /// No suitable XQuery processor available.
    ///
    /// Raised by fn:load-xquery-module if no XQuery processor is available
    /// supporting the requested XQuery version (or if none is available at
    /// all).
    #[error("No suitable XQuery processor available")]
    FOQM0006,
    /// Invalid value for cast/constructor.
    ///
    /// A general-purpose error raised when casting, if a cast between two
    /// datatypes is allowed in principle, but the supplied value cannot be
    /// converted: for example when attempting to cast the string "nine" to an
    /// integer.
    #[error("Invalid value for cast/constructor")]
    FORG0001,
    /// Invalid argument to fn:resolve-uri().
    ///
    /// Raised when either argument to fn:resolve-uri is not a valid URI/IRI.
    #[error("Invalid argument to fn:resolve-uri()")]
    FORG0002,
    /// fn:zero-or-one called with a sequence containing more than one item.
    ///
    /// Raised by fn:zero-or-one if the supplied value contains more than one item.
    #[error("fn:zero-or-one called with a sequence containing more than one item")]
    FORG0003,
    /// fn:one-or-more called with a sequence containing no items.
    ///
    /// Raised by fn:one-or-more if the supplied value is an empty sequence.
    #[error("fn:one-or-more called with a sequence containing no items")]
    FORG0004,
    /// fn:exactly-one called with a sequence containing zero or more than one item.
    ///
    /// Raised by fn:exactly-one if the supplied value is not a singleton sequence.
    #[error("fn:exactly-one called with a sequence containing zero or more than one item")]
    FORG0005,
    /// Invalid argument type.
    ///
    /// Raised by functions such as fn:max, fn:min, fn:avg, fn:sum if the
    /// supplied sequence contains values inappropriate to this function.
    #[error("Invalid argument type")]
    FORG0006,
    /// The two arguments to fn:dateTime have inconsistent timezones.
    ///
    /// Raised by fn:dateTime if the two arguments both have timezones and the
    /// timezones are different.
    #[error("The two arguments to fn:dateTime have inconsistent timezones")]
    FORG0008,
    /// Error in resolving a relative URI against a base URI in fn:resolve-uri.
    ///
    /// A catch-all error for fn:resolve-uri, recognizing that the
    /// implementation can choose between a variety of algorithms and that some
    /// of these may fail for a variety of reasons.
    #[error("Error in resolving a relative URI against a base URI in fn:resolve-uri")]
    FORG0009,
    /// Invalid date/time.
    ///
    /// Raised when the input to fn:parse-ietf-date does not match the
    /// prescribed grammar, or when it represents an invalid date/time such as
    /// 31 February.
    #[error("Invalid date/time")]
    FORG0010,
    /// Invalid regular expression flags.
    ///
    /// Raised by regular expression functions such as fn:matches and
    /// fn:replace if the regular expression flags contain a character other
    /// than i, m, q, s, or x.
    #[error("Invalid regular expression flags")]
    FORX0001,
    /// Invalid regular expression.
    ///
    /// Raised by regular expression functions such as fn:matches and
    /// fn:replace if the regular expression is syntactically invalid.
    #[error("Invalid regular expression")]
    FORX0002,
    /// Regular expression matches zero-length string.
    ///
    /// For functions such as fn:replace and fn:tokenize, raises an error if
    /// the supplied regular expression is capable of matching a zero length
    /// string.
    #[error("Regular expression matches zero-length string")]
    FORX0003,
    /// Invalid replacement string.
    ///
    /// Raised by fn:replace to report errors in the replacement string.
    #[error("Invalid replacement string")]
    FORX0004,
    /// Argument to fn:data() contains a node that does not have a typed value.
    ///
    /// Raised by fn:data, or by implicit atomization, if applied to a node
    /// with no typed value, the main example being an element validated
    /// against a complex type that defines it to have element-only content.
    #[error("Argument to fn:data() contains a node without a typed value")]
    FOTY0012,
    /// The argument to fn:data() contains a function item.
    ///
    /// Raised by fn:data, or by implicit atomization, if the sequence to be
    /// atomized contains a function item.
    #[error("Argument to fn:data() contains a function item")]
    FOTY0013,
    /// The argument to fn:string() is a function item.
    ///
    /// Raised by fn:string, or by implicit string conversion, if the input
    /// sequence contains a function item.
    #[error("Argument to fn:string() is a function item")]
    FOTY0014,
    /// An argument to fn:deep-equal() contains a function item.
    ///
    /// Raised by fn:deep-equal if either input sequence contains a function
    /// item.
    #[error("Argument to fn:deep-equal() contains a function item")]
    FOTY0015,
    /// Invalid $href argument to fn:unparsed-text() (etc.)
    ///
    /// A dynamic error is raised if the $href argument contains a fragment
    /// identifier, or if it cannot be used to retrieve a resource containing
    /// text.
    #[error("Invalid $href argument to fn:unparsed-text()")]
    FOUT1170,
    /// Cannot decode resource retrieved by fn:unparsed-text() (etc.)
    ///
    /// A dynamic error is raised if the retrieved resource contains octets
    /// that cannot be decoded into Unicode ·characters· using the specified
    /// encoding, or if the resulting characters are not permitted XML
    /// characters. This includes the case where the processor does not support
    /// the requested encoding.
    #[error("Cannot decode resource retrieved by fn:unparsed-text()")]
    FOUT1190,
    /// Cannot infer encoding of resource retrieved by fn:unparsed-text()
    /// (etc.)
    ///
    /// A dynamic error is raised if $encoding is absent and the processor
    /// cannot infer the encoding using external information and the encoding
    /// is not UTF-8.
    #[error("Cannot infer encoding of resource retrieved by fn:unparsed-text()")]
    FOUT1200,
    /// No suitable XSLT processor available
    ///
    /// A dynamic error is raised if no XSLT processor suitable for evaluating
    /// a call on fn:transform is available.
    #[error("No suitable XSLT processor available")]
    FOXT0001,
    /// Invalid parameters to XSLT transformation
    ///
    /// A dynamic error is raised if the parameters supplied to fn:transform
    /// are invalid, for example if two mutually-exclusive parameters are
    /// supplied. If a suitable XSLT error code is available (for example in
    /// the case where the requested initial-template does not exist in the
    /// stylesheet), that error code should be used in preference.
    #[error("Invalid parameters to XSLT transformation")]
    FOXT0002,
    /// XSLT transformation failed
    ///
    /// A dynamic error is raised if an XSLT transformation invoked using
    /// fn:transform fails with a static or dynamic error. The XSLT error code
    /// is used if available; this error code provides a fallback when no XSLT
    /// error code is returned, for example because the processor is an XSLT
    /// 1.0 processor.
    #[error("XSLT transformation failed")]
    FOXT0003,
    /// XSLT transformation has been disabled
    ///
    /// A dynamic error is raised if the fn:transform function is invoked when
    /// XSLT transformation (or a specific transformation option) has been
    /// disabled for security or other reasons.
    #[error("XSLT transformation has been disabled")]
    FOXT0004,
    /// XSLT output contains non-accepted characters
    ///
    /// A dynamic error is raised if the result of the fn:transform function
    /// contains characters available only in XML 1.1 and the calling processor
    /// cannot handle such characters.
    #[error("XSLT output contains non-accepted characters")]
    FOXT0006,

    #[error("type error")]
    TypeError,
}

pub type Result<T> = std::result::Result<T, Error>;
