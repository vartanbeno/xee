use std::fmt::Write;

use xee_interpreter::{error, sequence::Sequence};
use xee_xslt::evaluate;
use xot::Xot;

fn xml(xot: &Xot, sequence: Sequence) -> String {
    let mut f = String::new();

    for item in sequence.items() {
        f.write_str(
            &xot.to_string(item.unwrap().to_node().unwrap().xot_node())
                .unwrap(),
        )
        .unwrap();
    }
    f
}

#[test]
fn test_transform() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/"><a/></xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<a/>");
}

#[test]
fn test_transform_nested() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/"><a><b/><b/></a></xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<a><b/><b/></a>");
}

#[test]
fn test_transform_text_node() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/"><a>foo</a></xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<a>foo</a>");
}

#[test]
fn test_transform_nested_apply_templates() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><foo/><bar/></doc>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:apply-templates select="doc/*" /></o>
  </xsl:template>
  <xsl:template match="foo">
    <f/>
  </xsl:template>
  <xsl:template match="bar">
    <b/>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><f/><b/></o>");
}

#[test]
fn test_transform_value_of_select() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:value-of select="1 to 4" /></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>1 2 3 4</o>");
}

#[test]
fn test_transform_value_of_select_separator() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:value-of select="1 to 4" separator="|" /></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>1|2|3|4</o>");
}

#[test]
fn test_value_of_with_sequence_constructor() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:value-of>Hello</xsl:value-of></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>Hello</o>");
}

#[test]
fn test_transform_local_variable() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <xsl:variable name="foo" select="'FOO'"/>
    <o><xsl:value-of select="$foo"/></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o>FOO</o>");
}

#[test]
fn test_transform_local_variable_shadow() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <xsl:variable name="foo" select="'FOO'"/>
    <xsl:variable name="foo" select="'BAR'"/>
    <o><xsl:value-of select="$foo"/></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o>BAR</o>");
}

#[test]
fn test_transform_local_variable_from_sequence_constructor() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <xsl:variable name="foo"><b>B</b></xsl:variable>
    <o><xsl:value-of select="$foo"/></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o>B</o>");
}

#[test]
fn test_transform_if_true() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:if test="1"><foo/></xsl:if></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o><foo/></o>");
}

#[test]
fn test_transform_if_false() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:if test="0"><foo/></xsl:if></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o/>");
}

#[test]
fn test_transform_choose_when() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="1"><foo/></xsl:when>
      <xsl:otherwise><bar/></xsl:otherwise>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o><foo/></o>");
}

#[test]
fn test_transform_choose_otherwise() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="0"><foo/></xsl:when>
      <xsl:otherwise><bar/></xsl:otherwise>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o><bar/></o>");
}

#[test]
fn test_transform_choose_when_false_no_otherwise() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="0"><foo/></xsl:when>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o/>");
}

#[test]
fn test_transform_multiple_when() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="0"><foo/></xsl:when>
      <xsl:when test="1"><bar/></xsl:when>
      <xsl:otherwise><baz/></xsl:otherwise>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o><bar/></o>");
}

#[test]
fn test_transform_multiple_when_with_otherwise() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="0"><foo/></xsl:when>
      <xsl:when test="0"><bar/></xsl:when>
      <xsl:otherwise><baz/></xsl:otherwise>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(xml(&xot, output), "<o><baz/></o>");
}

#[test]
fn test_basic_for_each() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><foo/><foo/><foo/></doc>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:for-each select="doc/foo"><bar/></xsl:for-each></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><bar/><bar/><bar/></o>");
}

#[test]
fn test_for_each_context() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><foo>0</foo><foo>1</foo><foo>2</foo></doc>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:for-each select="doc/foo">
      <bar><xsl:value-of select="string()"/></bar>
    </xsl:for-each></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(
        xml(&xot, output),
        "<o><bar>0</bar><bar>1</bar><bar>2</bar></o>"
    );
}

#[test]
fn test_copy_empty_sequence() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:copy select="()"/></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o/>");
}

#[test]
fn test_copy_not_one_item_fails() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3" >
  <xsl:template match="/">
    <o><xsl:copy select="(1, 2)"/></o>
  </xsl:template>
</xsl:transform>"#,
    );
    // TODO: check the right error value
    assert!(matches!(output, error::SpannedResult::Err(_)));
}

#[test]
fn test_copy_atom() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
                 <xsl:template match="/">
                   <xsl:variable name="foo"><xsl:copy select="1"/></xsl:variable>
                   <o><xsl:value-of select="string($foo)"/></o>
                 </xsl:template>
              </xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>1</o>");
}

#[test]
fn test_copy_function() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
                 <xsl:template match="/">
                   <xsl:variable name="foo"><xsl:copy select="function() { 1 }"/></xsl:variable>
                   <o><xsl:value-of select="string($foo)"/></o>
                 </xsl:template>
              </xsl:transform>"#,
    );
    // this is an error as we try to atomize a function
    assert!(matches!(
        output,
        error::SpannedResult::Err(error::SpannedError {
            error: error::Error::FOTY0014,
            span: _
        })
    ));
}

#[test]
fn test_copy_text() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc>content</doc>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
                 <xsl:template match="/">
                   <xsl:variable name="foo"><xsl:copy select="doc/child::node()" /></xsl:variable>
                   <o><xsl:value-of select="string($foo)"/></o>
                 </xsl:template>
              </xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>content</o>");
}

#[test]
fn test_copy_element() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><p>Content</p></doc>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
                 <xsl:template match="/">
                   <o><xsl:copy select="doc/*" /></o>
                 </xsl:template>
              </xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><p/></o>");
}

#[test]
fn test_copy_element_with_sequence_constructor() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><p>Content</p></doc>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
                 <xsl:template match="/">
                   <o><xsl:copy select="doc/*">Constructed</xsl:copy></o>
                 </xsl:template>
              </xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><p>Constructed</p></o>");
}

#[test]
fn test_copy_of_atom() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o>
      <xsl:variable name="foo"><xsl:copy-of select="'foo'" /></xsl:variable>
      <xsl:value-of select="string($foo)"/>
    </o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>foo</o>");
}

#[test]
fn test_copy_of_node() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><foo>FOO</foo></doc>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o>
      <xsl:copy-of select="/doc/foo" />
    </o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><foo>FOO</foo></o>");
}

#[test]
fn test_sequence() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:value-of><xsl:sequence select="1 to 4" /></xsl:value-of></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>1 2 3 4</o>");
}

#[test]
fn test_complex_content_single_string() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o>
      <xsl:sequence select="'foo'" />
    </o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>foo</o>");
}

#[test]
fn test_complex_content_multiple_strings() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o>
      <xsl:sequence select="('foo', 'bar')" />
    </o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o>foo bar</o>");
}

#[test]
fn test_complex_content_xml_and_atomic() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o>
      <xsl:sequence select="('foo', 'bar')" />
      <hello>Hello</hello>
      <xsl:sequence select="('baz', 'qux')" />
    </o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(
        xml(&xot, output),
        "<o>foo bar<hello>Hello</hello>baz qux</o>"
    );
}

#[test]
fn test_function_item_in_complex_content() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc/>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:sequence select="function() { 1 }" /></o>
  </xsl:template>
</xsl:transform>"#,
    );

    assert!(matches!(
        output,
        error::SpannedResult::Err(error::SpannedError {
            error: error::Error::XTDE0450,
            span: _
        })
    ));
}

#[test]
fn test_source_nodes_complex_content() {
    let mut xot = Xot::new();
    // try this twice, so that we verify no mutation of source takes place and
    // source code nodes are properly copied
    let output = evaluate(
        &mut xot,
        "<doc><hello>Hello</hello></doc>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o>
      <xsl:sequence select="/doc/hello" />
      <xsl:sequence select="/doc/hello" />
    </o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(
        xml(&xot, output),
        "<o><hello>Hello</hello><hello>Hello</hello></o>"
    );
}

#[test]
fn test_transform_predicate() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        "<doc><foo>1</foo><foo>2</foo></doc>",
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:apply-templates select="doc/*" /></o>
  </xsl:template>
  <xsl:template match="foo[2]">
    <found><xsl:value-of select="string()" /></found>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><found>2</found></o>");
}

#[test]
fn test_transform_predicate_with_attribute() {
    let mut xot = Xot::new();
    let output = evaluate(
        &mut xot,
        r#"<doc><foo>1</foo><foo bar="BAR">2</foo></doc>"#,
        r#"
<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3">
  <xsl:template match="/">
    <o><xsl:apply-templates select="doc/*" /></o>
  </xsl:template>
  <xsl:template match="foo[@bar]">
    <found><xsl:value-of select="string()" /></found>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(xml(&xot, output), "<o><found>2</found></o>");
}
