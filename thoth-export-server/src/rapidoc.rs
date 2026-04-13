//! Utility module to generate a RapiDoc interface

/// Generate the HTML source to show a RapiDoc interface
pub fn rapidoc_source(openapi_spec: &str) -> String {
    let favicon_base_url = "https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent";
    format!(
        r##"
<!DOCTYPE html>
<html>
    <head>
        <title>Thoth Metadata Export API Documentation</title>
        <meta charset="utf-8">
        <link rel="shortcut icon" href="{favicon_base_url}/favicon.ico" />
        <link rel="apple-touch-icon" sizes="57x57" href="{favicon_base_url}/apple-icon-57x57.png">
        <link rel="apple-touch-icon" sizes="60x60" href="{favicon_base_url}/apple-icon-60x60.png">
        <link rel="apple-touch-icon" sizes="72x72" href="{favicon_base_url}/apple-icon-72x72.png">
        <link rel="apple-touch-icon" sizes="76x76" href="{favicon_base_url}/apple-icon-76x76.png">
        <link rel="apple-touch-icon" sizes="114x114" href="{favicon_base_url}/apple-icon-114x114.png">
        <link rel="apple-touch-icon" sizes="120x120" href="{favicon_base_url}/apple-icon-120x120.png">
        <link rel="apple-touch-icon" sizes="144x144" href="{favicon_base_url}/apple-icon-144x144.png">
        <link rel="apple-touch-icon" sizes="152x152" href="{favicon_base_url}/apple-icon-152x152.png">
        <link rel="apple-touch-icon" sizes="180x180" href="{favicon_base_url}/apple-icon-180x180.png">
        <link rel="icon" type="image/png" sizes="192x192"  href="{favicon_base_url}/android-icon-192x192.png">
        <link rel="icon" type="image/png" sizes="32x32" href="{favicon_base_url}/favicon-32x32.png">
        <link rel="icon" type="image/png" sizes="96x96" href="{favicon_base_url}/favicon-96x96.png">
        <link rel="icon" type="image/png" sizes="16x16" href="{favicon_base_url}/favicon-16x16.png">
        <link rel="manifest" href="{favicon_base_url}/manifest.json">
        <meta name="msapplication-TileColor" content="#fff3dc">
        <meta name="msapplication-TileImage" content="{favicon_base_url}/ms-icon-144x144.png">
        <meta name="theme-color" content="#fff3dc">
        <script src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
        <style>
            rapi-doc::part(anchor){{ color: #6e4f7f; }}
            rapi-doc::part(section-overview-title){{ color: #3c3c3b; }}
            rapi-doc::part(section-tag-title){{ color: #3c3c3b; }}
            rapi-doc::part(label-operation-method){{ color: #52a46a; }}
            rapi-doc::part(section-navbar-tag){{ color: #52a46a; }}
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
        primary-color = "#6e4f7f"
        nav-bg-color = "#fff3dc"
        bg-color = "#fffcf2"
        text-color = "#3c3c3b"
        font-size = "large"
      >
        <img
            slot="nav-logo"
            style="width:3.5em"
            src="https://cdn.thoth.pub/THOTH_Head.png"
          />
      </rapi-doc>
    </body>
</html>
"##
    )
}
