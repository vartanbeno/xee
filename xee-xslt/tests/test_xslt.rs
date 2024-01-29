use xee_xslt::evaluate;

#[test]
fn test_transform() {
    let output = evaluate(
            "<doc/>",
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><a/></xsl:template></xsl:transform>"#,
        ).unwrap();
    assert_eq!(output.to_string(), "<a/>");
}

#[test]
fn test_transform_nested() {
    let output = evaluate(
            "<doc/>",
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><a><b/><b/></a></xsl:template></xsl:transform>"#,
        ).unwrap();
    assert_eq!(output.to_string(), "<a><b/><b/></a>");
}

#[test]
fn test_transform_nested_apply_templates() {
    let output = evaluate(
        "<doc><foo/><bar/></doc>",
        r#"<xsl:transform version="3" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
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
    assert_eq!(output.to_string(), "<o><f/><b/></o>");
}

#[test]
fn test_transform_value_of_select() {
    let output = evaluate(
        "<doc/>",
        r#"
<xsl:transform version="3" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:value-of select="1 to 4" /></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(output.to_string(), "<o>1 2 3 4</o>");
}

#[test]
fn test_transform_value_of_select_separator() {
    let output = evaluate(
        "<doc/>",
        r#"
<xsl:transform version="3" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:value-of select="1 to 4" separator="|" /></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();
    assert_eq!(output.to_string(), "<o>1|2|3|4</o>");
}

#[test]
fn test_transform_local_variable() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <xsl:variable name="foo" select="'FOO'"/>
    <o><xsl:value-of select="$foo"/></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o>FOO</o>");
}

#[test]
fn test_transform_local_variable_shadow() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <xsl:variable name="foo" select="'FOO'"/>
    <xsl:variable name="foo" select="'BAR'"/>
    <o><xsl:value-of select="$foo"/></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o>BAR</o>");
}

#[test]
fn test_transform_if_true() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:if test="1"><foo/></xsl:if></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o><foo/></o>");
}

#[test]
fn test_transform_if_false() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:if test="0"><foo/></xsl:if></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o/>");
}

#[test]
fn test_transform_choose_when() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="1"><foo/></xsl:when>
      <xsl:otherwise><bar/></xsl:otherwise>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o><foo/></o>");
}

#[test]
fn test_transform_choose_otherwise() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="0"><foo/></xsl:when>
      <xsl:otherwise><bar/></xsl:otherwise>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o><bar/></o>");
}

#[test]
fn test_transform_choose_when_false_no_otherwise() {
    let output = evaluate(
        "<doc/>",
        r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <o><xsl:choose>
      <xsl:when test="0"><foo/></xsl:when>
    </xsl:choose></o>
  </xsl:template>
</xsl:transform>"#,
    )
    .unwrap();

    assert_eq!(output.to_string(), "<o/>");
}
