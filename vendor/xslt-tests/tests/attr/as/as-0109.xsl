<?xml version="1.0" encoding="UTF-8"?>
<xslt:transform xmlns:xs="http://www.w3.org/2001/XMLSchema"
                xmlns:xslt="http://www.w3.org/1999/XSL/Transform"
                exclude-result-prefixes="xs"
                version="2.0">
<!-- Purpose: Test of two xsl:templates with @as="xs:anyAtomicType?", 
  				one returns a singleton sequence, the other an empty sequence. -->

   <xslt:output method="xml" encoding="UTF-8"/>

   <xslt:template match="/doc">
      <out>
         <xslt:call-template name="temp1"/>
         <xslt:call-template name="temp2"/>
      </out>
   </xslt:template>

   <xslt:template name="temp1" as="xs:anyAtomicType?">
	     <xslt:sequence select="(xs:duration('P1Y2M3DT10H30M23S'))"/>
   </xslt:template>

   <xslt:template name="temp2" as="xs:anyAtomicType?">
	     <xslt:sequence select="()"/>
   </xslt:template>
</xslt:transform>
