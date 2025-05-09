<?xml version="1.0" encoding="UTF-8"?>
<xslt:transform xmlns:xs="http://www.w3.org/2001/XMLSchema"
                xmlns:my="http://www.mytest.net"
                xmlns:xslt="http://www.w3.org/1999/XSL/Transform"
                exclude-result-prefixes="xs my"
                version="2.0">
<!-- Purpose: Test with global xsl:variable without @select, where sequence constructor 
  				contains a typed value of the same type as specified in @as. 
				Values for @as tested: all built-in primitive types, xs:integer, 
				xs:dayTimeDuration, xs:yearMonthDuration  -->

   <xslt:output method="xml" encoding="UTF-8"/>

   <xslt:variable name="var1" as="xs:duration">
	     <xslt:value-of select="xs:duration('P1Y2M3DT10H30M23S')"/>
   </xslt:variable>

   <xslt:variable name="var2" as="xs:duration">
	     <xslt:value-of select="xs:duration('-P12M23DT0M59.123S')"/>
   </xslt:variable>

   <xslt:variable name="var3" as="xs:dateTime">
	     <xslt:value-of select="xs:dateTime('2000-01-16T00:00:00Z')"/>
   </xslt:variable>

   <xslt:variable name="var4" as="xs:dateTime">
	     <xslt:value-of select="xs:dateTime('-1999-05-31T13:20:00+14:00')"/>
   </xslt:variable>

   <xslt:variable name="var5" as="xs:time">
	     <xslt:value-of select="xs:time('13:20:00-05:00')"/>
   </xslt:variable>

   <xslt:variable name="var6" as="xs:time">
	     <xslt:value-of select="xs:time('00:00:00')"/>
   </xslt:variable>

   <xslt:variable name="var7" as="xs:date">
	     <xslt:value-of select="xs:date('-0012-12-03-05:00')"/>
   </xslt:variable>

   <xslt:variable name="var8" as="xs:date">
	     <xslt:value-of select="xs:date('1999-05-31')"/>
   </xslt:variable>

   <xslt:variable name="var9" as="xs:gYearMonth">
	     <xslt:value-of select="xs:gYearMonth('21999-05+14:00')"/>
   </xslt:variable>

   <xslt:variable name="var10" as="xs:gYearMonth">
	     <xslt:value-of select="xs:gYearMonth('1999-05')"/>
   </xslt:variable>

   <xslt:variable name="var11" as="xs:gYear">
	     <xslt:value-of select="xs:gYear('-0012-05:00')"/>
   </xslt:variable>

   <xslt:variable name="var12" as="xs:gYear">
	     <xslt:value-of select="xs:gYear('2005')"/>
   </xslt:variable>

   <xslt:variable name="var13" as="xs:gMonthDay">
	     <xslt:value-of select="xs:gMonthDay('--05-31')"/>
   </xslt:variable>

   <xslt:variable name="var14" as="xs:gMonthDay">
	     <xslt:value-of select="xs:gMonthDay('--05-31+14:00')"/>
   </xslt:variable>

   <xslt:variable name="var15" as="xs:gDay">
	     <xslt:value-of select="xs:gDay('---31+14:00')"/>
   </xslt:variable>

   <xslt:variable name="var16" as="xs:gDay">
	     <xslt:value-of select="xs:gDay('---31Z')"/>
   </xslt:variable>

   <xslt:variable name="var17" as="xs:gMonth">
	     <xslt:value-of select="xs:gMonth('--05+14:00')"/>
   </xslt:variable>

   <xslt:variable name="var18" as="xs:gMonth">
	     <xslt:value-of select="xs:gMonth('--05')"/>
   </xslt:variable>

   <xslt:variable name="var19" as="xs:boolean">
	     <xslt:value-of select="xs:boolean('true')"/>
   </xslt:variable>

   <xslt:variable name="var20" as="xs:base64Binary">
	     <xslt:value-of select="xs:base64Binary('PM24')"/>
   </xslt:variable>

   <xslt:variable name="var21" as="xs:hexBinary">
	     <xslt:value-of select="xs:hexBinary('07FF')"/>
   </xslt:variable>

   <xslt:variable name="var22" as="xs:float">
	     <xslt:value-of select="xs:float(1.1234E99)"/>
   </xslt:variable>

   <xslt:variable name="var23" as="xs:float">
	     <xslt:value-of select="xs:float(-1.75e-3)"/>
   </xslt:variable>

   <xslt:variable name="var24" as="xs:decimal">
	     <xslt:value-of select="xs:decimal(-3.420100)"/>
   </xslt:variable>

   <xslt:variable name="var25" as="xs:decimal">
	     <xslt:value-of select="xs:decimal(7.8899)"/>
   </xslt:variable>

   <xslt:variable name="var26" as="xs:decimal">
	     <xslt:value-of select="xs:decimal(0.0)"/>
   </xslt:variable>

   <xslt:variable name="var27" as="xs:double">
	     <xslt:value-of select="xs:double(5.4321E-1001)"/>
   </xslt:variable>

   <xslt:variable name="var28" as="xs:double">
	     <xslt:value-of select="xs:double(-0.0E0)"/>
   </xslt:variable>

   <xslt:variable name="var29" as="xs:double">
	     <xslt:value-of select="xs:double('INF')"/>
   </xslt:variable>

   <xslt:variable name="var30" as="xs:integer">
	     <xslt:value-of select="xs:integer(43)"/>
   </xslt:variable>

   <xslt:variable name="var31" as="xs:integer">
	     <xslt:value-of select="xs:integer(-300)"/>
   </xslt:variable>

   <xslt:variable name="var32" as="xs:anyURI">
	     <xslt:value-of select="xs:anyURI('http://www.mytypetest.org')"/>
   </xslt:variable>

   <xslt:variable name="var33" as="xs:QName">
	     <xslt:sequence select="xs:QName('my:local')"/>
   </xslt:variable>

   <xslt:variable name="var34" as="xs:dayTimeDuration">
	     <xslt:value-of select="xs:dayTimeDuration('PT99.999S')"/>
   </xslt:variable>

   <xslt:variable name="var35" as="xs:dayTimeDuration">
	     <xslt:value-of select="xs:dayTimeDuration('-PT100M')"/>
   </xslt:variable>

   <xslt:variable name="var36" as="xs:yearMonthDuration">
	     <xslt:value-of select="xs:yearMonthDuration('P1Y2M')"/>
   </xslt:variable>

   <xslt:variable name="var37" as="xs:yearMonthDuration">
	     <xslt:value-of select="xs:yearMonthDuration('-P21M')"/>
   </xslt:variable>

   <xslt:variable name="var38" as="xs:string">
	     <xslt:value-of select="xs:string('hello')"/>
   </xslt:variable>

   <xslt:template match="/doc">
      <out>
         
         <var1>
            <xslt:value-of select="$var1 instance of xs:duration"/>
         </var1>
         
         <var2>
            <xslt:value-of select="$var2 instance of xs:duration"/>
         </var2>
         
         <var3>
            <xslt:value-of select="$var3 instance of xs:dateTime"/>
         </var3>
         
         <var4>
            <xslt:value-of select="$var4 instance of xs:dateTime"/>
         </var4>
         
         <var5>
            <xslt:value-of select="$var5 instance of xs:time"/>
         </var5>
         
         <var6>
            <xslt:value-of select="$var6 instance of xs:time"/>
         </var6>
         
         <var7>
            <xslt:value-of select="$var7 instance of xs:date"/>
         </var7>
         
         <var8>
            <xslt:value-of select="$var8 instance of xs:date"/>
         </var8>
         
         <var9>
            <xslt:value-of select="$var9 instance of xs:gYearMonth"/>
         </var9>
         
         <var10>
            <xslt:value-of select="$var10 instance of xs:gYearMonth"/>
         </var10>
         
         <var11>
            <xslt:value-of select="$var11 instance of xs:gYear"/>
         </var11>
         
         <var12>
            <xslt:value-of select="$var12 instance of xs:gYear"/>
         </var12>
         
         <var13>
            <xslt:value-of select="$var13 instance of xs:gMonthDay"/>
         </var13>
         
         <var14>
            <xslt:value-of select="$var14 instance of xs:gMonthDay"/>
         </var14>
         
         <var15>
            <xslt:value-of select="$var15 instance of xs:gDay"/>
         </var15>
         
         <var16>
            <xslt:value-of select="$var16 instance of xs:gDay"/>
         </var16>
         
         <var17>
            <xslt:value-of select="$var17 instance of xs:gMonth"/>
         </var17>
         
         <var18>
            <xslt:value-of select="$var18 instance of xs:gMonth"/>
         </var18>
         
         <var19>
            <xslt:value-of select="$var19 instance of xs:boolean"/>
         </var19>
         
         <var20>
            <xslt:value-of select="$var20 instance of xs:base64Binary"/>
         </var20>
         
         <var21>
            <xslt:value-of select="$var21 instance of xs:hexBinary"/>
         </var21>
         
         <var22>
            <xslt:value-of select="$var22 instance of xs:float"/>
         </var22>
         
         <var23>
            <xslt:value-of select="$var23 instance of xs:float"/>
         </var23>
         
         <var24>
            <xslt:value-of select="$var24 instance of xs:decimal"/>
         </var24>
         
         <var25>
            <xslt:value-of select="$var25 instance of xs:decimal"/>
         </var25>
         
         <var26>
            <xslt:value-of select="$var26 instance of xs:decimal"/>
         </var26>
         
         <var27>
            <xslt:value-of select="$var27 instance of xs:double"/>
         </var27>
         
         <var28>
            <xslt:value-of select="$var28 instance of xs:double"/>
         </var28>
         
         <var29>
            <xslt:value-of select="$var29 instance of xs:double"/>
         </var29>
         
         <var30>
            <xslt:value-of select="$var30 instance of xs:integer"/>
         </var30>
         
         <var31>
            <xslt:value-of select="$var31 instance of xs:integer"/>
         </var31>
         
         <var32>
            <xslt:value-of select="$var32 instance of xs:anyURI"/>
         </var32>
         
         <var33>
            <xslt:value-of select="$var33 instance of xs:QName"/>
         </var33>
         
         <var34>
            <xslt:value-of select="$var34 instance of xs:dayTimeDuration"/>
         </var34>
         
         <var35>
            <xslt:value-of select="$var35 instance of xs:dayTimeDuration"/>
         </var35>
         
         <var36>
            <xslt:value-of select="$var36 instance of xs:yearMonthDuration"/>
         </var36>
         
         <var37>
            <xslt:value-of select="$var37 instance of xs:yearMonthDuration"/>
         </var37>
         
         <var38>
            <xslt:value-of select="$var38 instance of xs:string"/>
         </var38>
      </out>
   </xslt:template>
</xslt:transform>
