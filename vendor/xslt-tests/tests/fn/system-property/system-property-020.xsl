<?xml version="1.0" encoding="UTF-8"?>
<xslt:transform xmlns:xs="http://www.w3.org/2001/XMLSchema"
                xmlns:xslt="http://www.w3.org/1999/XSL/Transform"
                exclude-result-prefixes="xs"
                version="2.0">
<!-- Purpose: Test that unknown system property returns empty string.-->

   <xslt:template match="/">
      <out>
         <a><xslt:value-of select="system-property('xslt:is-the-best-thing-ever')"/></a>
         <a><xslt:value-of select="system-property('xs:is-the-worst-thing-ever')"/></a>
         <a><xslt:value-of select="system-property('verisimilitude')"/></a>
      </out>
   </xslt:template>
</xslt:transform>
