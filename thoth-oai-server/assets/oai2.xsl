<?xml version="1.0" encoding="utf-8"?>
<xsl:stylesheet
    version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:oai="http://www.openarchives.org/OAI/2.0/"
    xmlns:id="http://www.openarchives.org/OAI/2.0/oai-identifier"
    xmlns:thoth="https://thoth.pub/oai/"
    xmlns:oai_dc="http://www.openarchives.org/OAI/2.0/oai_dc/"
    xmlns:oaire="http://namespace.openaire.eu/schema/oaire/"
    xmlns:marc="http://www.loc.gov/MARC21/slim"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    exclude-result-prefixes="oai id thoth oai_dc oaire marc dc"
>
<xsl:output method="html" encoding="utf-8" omit-xml-declaration="yes"/>

<xsl:template match="/">
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <title>Thoth OAI-PMH Browser</title>
    <style>
@font-face {
  font-family: "Economica";
  src: url(https://cdn.thoth.pub/fonts/Economica/Economica-Bold.ttf) format("truetype");
  font-display: swap;
}
@font-face {
  font-family: "Open Sans";
  src: url(https://cdn.thoth.pub/fonts/Open_Sans/OpenSans-VariableFont.ttf) format("truetype");
  font-display: swap;
}
:root {
  --thoth-primary: #6e4f7f;
  --thoth-secondary: #52a46a;
  --thoth-accent: #ffdd75;
  --thoth-body: #3c3c3b;
  --thoth-soft: #fff4e8;
  --thoth-border: #d9c7aa;
}
* {
  box-sizing: border-box;
}
body {
  margin: 0;
  font-family: "Open Sans", Arial, sans-serif;
  color: var(--thoth-body);
  background: linear-gradient(180deg, #fffdf8 0%, #fff9f0 100%);
}
a {
  color: var(--thoth-primary);
}
.shell {
  max-width: 1140px;
  margin: 0 auto;
  padding: 1.5rem 1rem 3rem;
}
.brand-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 1rem;
  border: 1px solid var(--thoth-border);
  border-radius: 1rem;
  background: #ffffff;
  padding: 1rem 1.25rem;
  box-shadow: 0 12px 30px rgba(60, 60, 59, 0.08);
}
.brand-logo {
  width: 220px;
  max-width: 100%;
  height: auto;
}
.brand-title {
  margin: 0;
  font-family: "Economica", "Open Sans", Arial, sans-serif;
  font-size: 2.4rem;
  line-height: 1;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}
.brand-subtitle {
  margin: 0.35rem 0 0;
  font-size: 0.95rem;
  color: #5f5f5e;
}
.quicklinks {
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
  margin: 1rem 0 0;
  padding: 0;
}
.quicklinks a {
  display: inline-block;
  border-radius: 999px;
  border: 1px solid transparent;
  background: rgba(110, 79, 127, 0.12);
  color: var(--thoth-primary);
  text-decoration: none;
  font-size: 0.85rem;
  font-weight: 700;
  padding: 0.4rem 0.85rem;
}
.quicklinks a:hover {
  border-color: var(--thoth-primary);
  background: rgba(110, 79, 127, 0.18);
}
.overview {
  margin-top: 1.1rem;
  border: 1px solid var(--thoth-border);
  border-radius: 0.9rem;
  background: #ffffff;
  padding: 1rem 1.25rem;
}
.section {
  margin-top: 1rem;
  border: 1px solid var(--thoth-border);
  border-radius: 0.9rem;
  background: #ffffff;
  padding: 1rem 1.25rem;
}
.section-title {
  margin: 0;
  font-family: "Economica", "Open Sans", Arial, sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--thoth-primary);
}
.section-subtitle {
  margin: 0.75rem 0 0.45rem;
  font-family: "Economica", "Open Sans", Arial, sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--thoth-secondary);
}
.intro {
  margin: 0.45rem 0 0;
  color: #5f5f5e;
}
.kv-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 0.8rem;
}
.kv-table th,
.kv-table td {
  border: 1px solid var(--thoth-border);
  padding: 0.55rem 0.7rem;
  text-align: left;
  vertical-align: top;
}
.kv-table th {
  width: 28%;
  background: var(--thoth-soft);
  font-size: 0.86rem;
}
.record-card {
  margin-top: 1rem;
  border: 1px solid var(--thoth-border);
  border-radius: 0.8rem;
  overflow: hidden;
}
.record-card > h3 {
  margin: 0;
  font-size: 1rem;
  color: #ffffff;
  background: linear-gradient(90deg, var(--thoth-primary) 0%, var(--thoth-secondary) 100%);
  padding: 0.65rem 0.85rem;
}
.record-card-body {
  background: #ffffff;
  padding: 0.9rem;
}
.pill {
  display: inline-block;
  border-radius: 999px;
  background: rgba(255, 221, 117, 0.26);
  border: 1px solid rgba(110, 79, 127, 0.2);
  color: var(--thoth-body);
  padding: 0.2rem 0.6rem;
  margin-right: 0.3rem;
  margin-bottom: 0.3rem;
  font-size: 0.78rem;
}
.meta-note {
  margin: 0.7rem 0 0;
  font-size: 0.9rem;
}
.xml-box {
  border: 1px solid var(--thoth-border);
  border-radius: 0.6rem;
  background: #fff;
  padding: 0.55rem;
  margin-top: 0.55rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 0.78rem;
  line-height: 1.35;
  overflow: auto;
}
.xml-node {
  padding-left: 1rem;
  white-space: nowrap;
}
.xml-tag {
  color: #7b355c;
  font-weight: 700;
}
.xml-attr {
  color: #244f8f;
}
.xml-text {
  color: #2e2e2d;
  white-space: normal;
}
.error-card {
  border: 1px solid #cb6b7d;
  border-radius: 0.8rem;
  background: rgba(241, 149, 168, 0.12);
  padding: 0.9rem 1rem;
  margin-top: 1rem;
}
.error-title {
  margin: 0;
  color: #8f2238;
  font-size: 1rem;
}
.error-text {
  margin: 0.4rem 0 0;
  color: #5f2734;
}
.footer {
  margin-top: 1.4rem;
  padding-top: 0.8rem;
  border-top: 1px solid var(--thoth-border);
  font-size: 0.85rem;
  color: #5f5f5e;
}
@media (max-width: 780px) {
  .brand-title {
    font-size: 1.9rem;
  }
  .kv-table th {
    width: 36%;
  }
}
    </style>
  </head>
  <body>
    <div class="shell">
      <header class="brand-bar">
        <img class="brand-logo" src="https://cdn.thoth.pub/THOTH_ColourPos.png" alt="Thoth logo"/>
        <div>
          <h1 class="brand-title">Thoth OAI-PMH Browser</h1>
          <p class="brand-subtitle">Human-friendly view of OAI-PMH XML responses.</p>
          <xsl:call-template name="quicklinks"/>
        </div>
      </header>
      <xsl:apply-templates select="/oai:OAI-PMH"/>
      <footer class="footer">
        <p>Rendered by Thoth's OAI stylesheet. Use your browser's "View Source" to inspect raw XML.</p>
      </footer>
    </div>
  </body>
</html>
</xsl:template>

<xsl:template name="quicklinks">
  <ul class="quicklinks">
    <li><a href="?verb=Identify">Identify</a></li>
    <li><a href="?verb=ListRecords&amp;metadataPrefix=oai_dc">ListRecords</a></li>
    <li><a href="?verb=ListIdentifiers&amp;metadataPrefix=oai_dc">ListIdentifiers</a></li>
    <li><a href="?verb=ListSets">ListSets</a></li>
    <li><a href="?verb=ListMetadataFormats">ListMetadataFormats</a></li>
  </ul>
</xsl:template>

<xsl:template match="/oai:OAI-PMH">
  <section class="overview">
    <h2 class="section-title">Response Overview</h2>
    <table class="kv-table">
      <tr><th>Response Date</th><td><xsl:value-of select="oai:responseDate"/></td></tr>
      <tr><th>Request URL</th><td><xsl:value-of select="oai:request"/></td></tr>
      <tr>
        <th>Verb</th>
        <td>
          <xsl:choose>
            <xsl:when test="string-length(oai:request/@verb) &gt; 0">
              <xsl:value-of select="oai:request/@verb"/>
            </xsl:when>
            <xsl:otherwise>unknown</xsl:otherwise>
          </xsl:choose>
        </td>
      </tr>
    </table>
  </section>
  <xsl:choose>
    <xsl:when test="oai:error">
      <xsl:apply-templates select="oai:error"/>
    </xsl:when>
    <xsl:otherwise>
      <xsl:apply-templates select="oai:Identify"/>
      <xsl:apply-templates select="oai:GetRecord"/>
      <xsl:apply-templates select="oai:ListRecords"/>
      <xsl:apply-templates select="oai:ListIdentifiers"/>
      <xsl:apply-templates select="oai:ListSets"/>
      <xsl:apply-templates select="oai:ListMetadataFormats"/>
    </xsl:otherwise>
  </xsl:choose>
</xsl:template>

<xsl:template match="oai:error">
  <section class="error-card">
    <h2 class="error-title">OAI Error: <xsl:value-of select="@code"/></h2>
    <p class="error-text"><xsl:value-of select="."/></p>
  </section>
</xsl:template>

<xsl:template match="oai:Identify">
  <section class="section">
    <h2 class="section-title">Identify</h2>
    <table class="kv-table">
      <tr><th>Repository Name</th><td><xsl:value-of select="oai:repositoryName"/></td></tr>
      <tr><th>Base URL</th><td><xsl:value-of select="oai:baseURL"/></td></tr>
      <tr><th>Protocol Version</th><td><xsl:value-of select="oai:protocolVersion"/></td></tr>
      <tr><th>Earliest Datestamp</th><td><xsl:value-of select="oai:earliestDatestamp"/></td></tr>
      <tr><th>Deleted Record Policy</th><td><xsl:value-of select="oai:deletedRecord"/></td></tr>
      <tr><th>Granularity</th><td><xsl:value-of select="oai:granularity"/></td></tr>
      <xsl:for-each select="oai:compression">
        <tr><th>Compression</th><td><xsl:value-of select="."/></td></tr>
      </xsl:for-each>
      <xsl:for-each select="oai:adminEmail">
        <tr><th>Admin Email</th><td><xsl:value-of select="."/></td></tr>
      </xsl:for-each>
    </table>
    <xsl:apply-templates select="oai:description/*"/>
  </section>
</xsl:template>

<xsl:template match="id:oai-identifier">
  <h3 class="section-subtitle">OAI Identifier</h3>
  <table class="kv-table">
    <tr><th>Scheme</th><td><xsl:value-of select="id:scheme"/></td></tr>
    <tr><th>Repository Identifier</th><td><xsl:value-of select="id:repositoryIdentifier"/></td></tr>
    <tr><th>Delimiter</th><td><xsl:value-of select="id:delimiter"/></td></tr>
    <tr><th>Sample Identifier</th><td><xsl:value-of select="id:sampleIdentifier"/></td></tr>
  </table>
</xsl:template>

<xsl:template match="thoth:repository">
  <h3 class="section-subtitle">Thoth Repository Metadata</h3>
  <table class="kv-table">
    <tr><th>Latest Datestamp</th><td><xsl:value-of select="thoth:latestDatestamp"/></td></tr>
    <tr><th>Rights Management</th><td><xsl:value-of select="thoth:rightsStatement"/></td></tr>
    <tr>
      <th>Rights URL</th>
      <td>
        <xsl:choose>
          <xsl:when test="string-length(normalize-space(thoth:rightsUri)) &gt; 0">
            <a href="{thoth:rightsUri}"><xsl:value-of select="thoth:rightsUri"/></a>
          </xsl:when>
          <xsl:otherwise>Not provided</xsl:otherwise>
        </xsl:choose>
      </td>
    </tr>
  </table>
</xsl:template>

<xsl:template match="oai:description/*" priority="-10">
  <h3 class="section-subtitle">Description (Additional Metadata)</h3>
  <div class="xml-box">
    <xsl:apply-templates select="." mode="xml-pretty"/>
  </div>
</xsl:template>

<xsl:template match="oai:GetRecord">
  <section class="section">
    <h2 class="section-title">GetRecord</h2>
    <xsl:apply-templates select="oai:record"/>
  </section>
</xsl:template>

<xsl:template match="oai:ListRecords">
  <section class="section">
    <h2 class="section-title">ListRecords</h2>
    <xsl:apply-templates select="oai:record"/>
    <xsl:apply-templates select="oai:resumptionToken"/>
  </section>
</xsl:template>

<xsl:template match="oai:ListIdentifiers">
  <section class="section">
    <h2 class="section-title">ListIdentifiers</h2>
    <xsl:apply-templates select="oai:header"/>
    <xsl:apply-templates select="oai:resumptionToken"/>
  </section>
</xsl:template>

<xsl:template match="oai:ListSets">
  <section class="section">
    <h2 class="section-title">ListSets</h2>
    <xsl:apply-templates select="oai:set"/>
    <xsl:apply-templates select="oai:resumptionToken"/>
  </section>
</xsl:template>

<xsl:template match="oai:set">
  <div class="record-card">
    <h3>Set: <xsl:value-of select="oai:setSpec"/></h3>
    <div class="record-card-body">
      <table class="kv-table">
        <tr><th>Set Spec</th><td><xsl:value-of select="oai:setSpec"/></td></tr>
        <tr><th>Set Name</th><td><xsl:value-of select="oai:setName"/></td></tr>
      </table>
      <xsl:apply-templates select="oai:setDescription"/>
    </div>
  </div>
</xsl:template>

<xsl:template match="oai:setDescription">
  <h4 class="section-subtitle">Set Description</h4>
  <div class="xml-box">
    <xsl:apply-templates select="*" mode="xml-pretty"/>
  </div>
</xsl:template>

<xsl:template match="oai:ListMetadataFormats">
  <section class="section">
    <h2 class="section-title">ListMetadataFormats</h2>
    <xsl:if test="string-length(normalize-space(/oai:OAI-PMH/oai:request/@identifier)) &gt; 0">
      <p class="intro">
        Formats for identifier:
        <strong><xsl:value-of select="/oai:OAI-PMH/oai:request/@identifier"/></strong>
      </p>
    </xsl:if>
    <xsl:apply-templates select="oai:metadataFormat"/>
  </section>
</xsl:template>

<xsl:template match="oai:metadataFormat">
  <div class="record-card">
    <h3>Metadata Format: <xsl:value-of select="oai:metadataPrefix"/></h3>
    <div class="record-card-body">
      <table class="kv-table">
        <tr>
          <th>metadataPrefix</th>
          <td>
            <span class="pill"><xsl:value-of select="oai:metadataPrefix"/></span>
            <a href="?verb=ListRecords&amp;metadataPrefix={oai:metadataPrefix}">ListRecords</a>
          </td>
        </tr>
        <tr><th>metadataNamespace</th><td><xsl:value-of select="oai:metadataNamespace"/></td></tr>
        <tr><th>schema</th><td><a href="{oai:schema}"><xsl:value-of select="oai:schema"/></a></td></tr>
      </table>
      <xsl:if test="string-length(normalize-space(/oai:OAI-PMH/oai:request/@identifier)) &gt; 0">
        <p class="meta-note">
          <a href="?verb=GetRecord&amp;metadataPrefix={oai:metadataPrefix}&amp;identifier={/oai:OAI-PMH/oai:request/@identifier}">
            View this record in <xsl:value-of select="oai:metadataPrefix"/>
          </a>
        </p>
      </xsl:if>
    </div>
  </div>
</xsl:template>

<xsl:template match="oai:record">
  <div class="record-card">
    <h3>Record: <xsl:value-of select="oai:header/oai:identifier"/></h3>
    <div class="record-card-body">
      <xsl:apply-templates select="oai:header"/>
      <xsl:apply-templates select="oai:metadata"/>
      <xsl:apply-templates select="oai:about"/>
    </div>
  </div>
</xsl:template>

<xsl:template match="oai:header">
  <h4 class="section-subtitle">Header</h4>
  <table class="kv-table">
    <tr>
      <th>Identifier</th>
      <td>
        <xsl:value-of select="oai:identifier"/>
        <xsl:text> </xsl:text>
        <a href="?verb=ListMetadataFormats&amp;identifier={oai:identifier}">Formats</a>
      </td>
    </tr>
    <tr><th>Datestamp</th><td><xsl:value-of select="oai:datestamp"/></td></tr>
    <xsl:for-each select="oai:setSpec">
      <tr>
        <th>Set Spec</th>
        <td>
          <xsl:value-of select="."/>
          <xsl:text> </xsl:text>
          <a href="?verb=ListIdentifiers&amp;metadataPrefix=oai_dc&amp;set={.}">Identifiers</a>
          <xsl:text> </xsl:text>
          <a href="?verb=ListRecords&amp;metadataPrefix=oai_dc&amp;set={.}">Records</a>
        </td>
      </tr>
    </xsl:for-each>
    <xsl:if test="@status='deleted'">
      <tr><th>Status</th><td>deleted</td></tr>
    </xsl:if>
  </table>
</xsl:template>

<xsl:template match="oai:metadata">
  <h4 class="section-subtitle">Metadata</h4>
  <xsl:choose>
    <xsl:when test="*">
      <xsl:apply-templates select="*"/>
    </xsl:when>
    <xsl:otherwise>
      <p class="meta-note">No metadata payload for this record.</p>
    </xsl:otherwise>
  </xsl:choose>
</xsl:template>

<xsl:template match="oai:about">
  <h4 class="section-subtitle">About</h4>
  <xsl:choose>
    <xsl:when test="*">
      <div class="xml-box">
        <xsl:apply-templates select="*" mode="xml-pretty"/>
      </div>
    </xsl:when>
    <xsl:otherwise>
      <p class="meta-note">No additional about metadata.</p>
    </xsl:otherwise>
  </xsl:choose>
</xsl:template>

<xsl:template match="oai_dc:dc">
  <div class="record-card">
    <h3>Dublin Core (oai_dc)</h3>
    <div class="record-card-body">
      <table class="kv-table">
        <xsl:apply-templates select="dc:*"/>
      </table>
    </div>
  </div>
</xsl:template>

<xsl:template match="dc:*">
  <xsl:variable name="value" select="normalize-space(.)"/>
  <tr>
    <th><xsl:value-of select="local-name()"/></th>
    <td>
      <xsl:choose>
        <xsl:when test="starts-with($value, 'http://') or starts-with($value, 'https://')">
          <a href="{$value}"><xsl:value-of select="$value"/></a>
        </xsl:when>
        <xsl:otherwise><xsl:value-of select="$value"/></xsl:otherwise>
      </xsl:choose>
    </td>
  </tr>
</xsl:template>

<xsl:template match="oaire:resource">
  <div class="record-card">
    <h3>OpenAIRE (oai_openaire)</h3>
    <div class="record-card-body">
      <div class="xml-box">
        <xsl:apply-templates select="." mode="xml-pretty"/>
      </div>
    </div>
  </div>
</xsl:template>

<xsl:template match="marc:record">
  <div class="record-card">
    <h3>MARCXML (marcxml)</h3>
    <div class="record-card-body">
      <div class="xml-box">
        <xsl:apply-templates select="." mode="xml-pretty"/>
      </div>
    </div>
  </div>
</xsl:template>

<xsl:template match="oai:metadata/*" priority="-10">
  <div class="record-card">
    <h3>Metadata (Unsupported Format)</h3>
    <div class="record-card-body">
      <div class="xml-box">
        <xsl:apply-templates select="." mode="xml-pretty"/>
      </div>
    </div>
  </div>
</xsl:template>

<xsl:template match="oai:resumptionToken">
  <div class="record-card">
    <h3>Resumption Token</h3>
    <div class="record-card-body">
      <xsl:choose>
        <xsl:when test="string-length(normalize-space(.)) &gt; 0">
          <p class="meta-note">More results are available.</p>
          <table class="kv-table">
            <xsl:if test="@expirationDate">
              <tr><th>expirationDate</th><td><xsl:value-of select="@expirationDate"/></td></tr>
            </xsl:if>
            <xsl:if test="@completeListSize">
              <tr><th>completeListSize</th><td><xsl:value-of select="@completeListSize"/></td></tr>
            </xsl:if>
            <xsl:if test="@cursor">
              <tr><th>cursor</th><td><xsl:value-of select="@cursor"/></td></tr>
            </xsl:if>
            <tr><th>token</th><td><xsl:value-of select="."/></td></tr>
            <tr>
              <th>resume</th>
              <td>
                <a href="?verb={/oai:OAI-PMH/oai:request/@verb}&amp;resumptionToken={.}">Resume Listing</a>
              </td>
            </tr>
          </table>
        </xsl:when>
        <xsl:otherwise>
          <p class="meta-note">End of list. This empty token marks a terminal page.</p>
        </xsl:otherwise>
      </xsl:choose>
    </div>
  </div>
</xsl:template>

<xsl:template match="*" mode="xml-pretty">
  <div class="xml-node">
    &lt;<span class="xml-tag"><xsl:value-of select="name()"/></span><xsl:apply-templates select="@*" mode="xml-pretty"/>&gt;
    <xsl:apply-templates select="node()" mode="xml-pretty"/>
    &lt;/<span class="xml-tag"><xsl:value-of select="name()"/></span>&gt;
  </div>
</xsl:template>

<xsl:template match="@*" mode="xml-pretty">
  <xsl:text> </xsl:text>
  <span class="xml-attr"><xsl:value-of select="name()"/></span>="<span class="xml-text"><xsl:value-of select="."/></span>"
</xsl:template>

<xsl:template match="text()[normalize-space(.) != '']" mode="xml-pretty">
  <span class="xml-text"><xsl:value-of select="normalize-space(.)"/></span>
</xsl:template>

<xsl:template match="text()" mode="xml-pretty"/>

</xsl:stylesheet>
