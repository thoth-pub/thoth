//! Utility module to generate a RapiDoc interface

/// Generate the HTML source to show a RapiDoc interface
pub fn rapidoc_source(openapi_spec: &str) -> String {
    format!(
        r##"
<!DOCTYPE html>
<html>
    <head>
        <title>Thoth Metadata Export API Documentation</title>
        <meta charset="utf-8">
        <link rel="shortcut icon" href="https://cdn.thoth.pub/favicon.ico" />
        <link rel="apple-touch-icon" sizes="57x57" href="https://cdn.thoth.pub/apple-icon-57x57.png">
        <link rel="apple-touch-icon" sizes="60x60" href="https://cdn.thoth.pub/apple-icon-60x60.png">
        <link rel="apple-touch-icon" sizes="72x72" href="https://cdn.thoth.pub/apple-icon-72x72.png">
        <link rel="apple-touch-icon" sizes="76x76" href="https://cdn.thoth.pub/apple-icon-76x76.png">
        <link rel="apple-touch-icon" sizes="114x114" href="https://cdn.thoth.pub/apple-icon-114x114.png">
        <link rel="apple-touch-icon" sizes="120x120" href="https://cdn.thoth.pub/apple-icon-120x120.png">
        <link rel="apple-touch-icon" sizes="144x144" href="https://cdn.thoth.pub/apple-icon-144x144.png">
        <link rel="apple-touch-icon" sizes="152x152" href="https://cdn.thoth.pub/apple-icon-152x152.png">
        <link rel="apple-touch-icon" sizes="180x180" href="https://cdn.thoth.pub/apple-icon-180x180.png">
        <link rel="icon" type="image/png" sizes="192x192"  href="https://cdn.thoth.pub/android-icon-192x192.png">
        <link rel="icon" type="image/png" sizes="32x32" href="https://cdn.thoth.pub/favicon-32x32.png">
        <link rel="icon" type="image/png" sizes="96x96" href="https://cdn.thoth.pub/favicon-96x96.png">
        <link rel="icon" type="image/png" sizes="16x16" href="https://cdn.thoth.pub/favicon-16x16.png">
        <link rel="manifest" href="https://cdn.thoth.pub/manifest.json">
        <meta name="msapplication-TileColor" content="#FFDD57">
        <meta name="msapplication-TileImage" content="https://cdn.thoth.pub/ms-icon-144x144.png">
        <meta name="theme-color" content="#FFDD57">
        <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
        <style>
            rapi-doc::part(anchor){{ color: #7b377b; }}
            rapi-doc::part(section-overview-title){{ color: #007944; }}
            rapi-doc::part(section-tag-title){{ color: #007944; }}
            rapi-doc::part(label-operation-method){{ color: #007944; }}
            rapi-doc::part(section-navbar-tag){{ color: #007944; }}
        </style>
    </head>
    <body>
      <rapi-doc
        spec-url="{openapi_spec}"
        render-style = "read"
        theme = "light"
        show-header = "false"
        allow-authentication = "false"
        allow-server-selection = "false"
        primary-color = "#8c3f8d"
        nav-bg-color = "#ffdd57"
        bg-color = "#fffcf2"
        text-color = "#111827"
        font-size = "large"
      >
        <img
            slot="nav-logo"
            style="width:6em"
            src="https://cdn.thoth.pub/thoth_logo.png"
          />
      </rapi-doc>
    </body>
</html>
"##
    )
}
