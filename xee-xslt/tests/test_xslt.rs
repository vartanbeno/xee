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
