//! Utility module to generate a RapiDoc interface

/// Generate the HTML source to show a RapiDoc interface
pub fn rapidoc_source(openapi_spec: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
      <title>Thoth API Documentation</title>
      <meta charset="utf-8">
      <link rel="shortcut icon" href="https://thoth.pub/favicon.ico" />
      <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
    </head>
    <body>
      <rapi-doc
        spec-url="{openapi_spec}"
        render-style = "read"
        theme = "light"
        show-header = "false"
        allow-authentication = "false"
        allow-server-selection = "false"
      >
        <img
            slot="nav-logo"
            style="width:6em"
            src="https://thoth.pub/img/thoth-logo.png"
          />
      </rapi-doc>
    </body>
</html>
"#,
        openapi_spec = openapi_spec
    )
}
