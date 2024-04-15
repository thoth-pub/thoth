//! Utility module to generate a GraphiQL interface

/// Generate the HTML source to show a GraphiQL interface
pub fn graphiql_source(graphql_endpoint_url: &str) -> String {
    let default_query = r##"# Welcome to Thoth's GraphQL API explorer (GraphiQL).
#
# GraphiQL is an in-browser tool for writing, validating, and
# testing GraphQL queries.
#
# Type queries into this side of the screen, and you will see intelligent
# typeaheads aware of the current GraphQL type schema and live syntax and
# validation errors highlighted within the text.
#
# Click on the QueryRoot in the Documentation Explorer (first icon) on
# the top left of the screen to navigate the API schema.
#
# GraphQL queries typically start with a "{" character. Lines that starts
# with a "#" are ignored.
#
# Run the following example Thoth GraphQL query:
#
#       Run Query:  Ctrl-Enter (or press the play button above)
#
{
    books(order: {field: PUBLICATION_DATE, direction: ASC}) {
       fullTitle
       doi
       publications {
            publicationType
            isbn
       }
       contributions {
            contributionType
            fullName
       }
    }
}
"##;

    format!(
        r##"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Thoth GraphQL API Documentation</title>
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
    <script crossorigin src="https://unpkg.com/react@18/umd/react.development.js"></script>
    <script crossorigin src="https://unpkg.com/react-dom@18/umd/react-dom.development.js"></script>
    <script src="https://unpkg.com/graphiql@3.2/graphiql.min.js" type="application/javascript"></script>
    <link rel="stylesheet" href="https://unpkg.com/graphiql@3.2/graphiql.min.css" />
    <script crossorigin src="https://unpkg.com/@graphiql/plugin-explorer@2/dist/index.umd.js"></script>
    <link rel="stylesheet" href="https://unpkg.com/@graphiql/plugin-explorer@2/dist/style.css"/>
    <style>
     .graphiql-container,
      .CodeMirror-info,
      .CodeMirror-lint-tooltip,
      .graphiql-dialog,
      .graphiql-dialog-overlay,
      .graphiql-tooltip,
      [data-radix-popper-content-wrapper] {{
        --color-primary: 298, 38%, 40%;
        --color-secondary: 154, 100%, 24%;
        --color-tertiary: 320, 95%, 43%;
        --color-info: 188, 100%, 36%;
        --color-warning: 24, 100%, 63%;
        --color-base: 46, 100%, 97%;
      }}
      @media (prefers-color-scheme: dark) {{
          body:not(.graphiql-light) .graphiql-container,
          body:not(.graphiql-light) .CodeMirror-info,
          body:not(.graphiql-light) .CodeMirror-lint-tooltip,
          body:not(.graphiql-light) .graphiql-dialog,
          body:not(.graphiql-light) .graphiql-dialog-overlay,
          body:not(.graphiql-light) .graphiql-tooltip,
          body:not(.graphiql-light) [data-radix-popper-content-wrapper] {{
            --color-primary: 298, 38%, 40%;
            --color-secondary: 154, 58%, 47%;
            --color-tertiary: 338, 100%, 67%;
            --color-info: 188, 100%, 36%;
            --color-warning: 24, 100%, 63%;
            --color-base: 221, 39%, 11%;
          }}
      }}
      body.graphiql-dark .graphiql-container,
      body.graphiql-dark .CodeMirror-info,
      body.graphiql-dark .CodeMirror-lint-tooltip,
      body.graphiql-dark .graphiql-dialog,
      body.graphiql-dark .graphiql-dialog-overlay,
      body.graphiql-dark .graphiql-tooltip,
      body.graphiql-dark [data-radix-popper-content-wrapper] {{
        --color-primary: 298, 38%, 40%;
        --color-secondary: 154, 58%, 47%;
        --color-tertiary: 338, 100%, 67%;
        --color-info: 188, 100%, 36%;
        --color-warning: 24, 100%, 63%;
        --color-base: 221, 39%, 11%;
      }}
      body {{
        height: 100%;
        margin: 0;
        width: 100%;
        overflow: hidden;
      }}

      #graphiql {{
        height: 100vh;
      }}
    </style>
  </head>

  <body>
    <div id="graphiql">Loading...</div>
    <script>
      const root = ReactDOM.createRoot(document.getElementById('graphiql'));
      const fetcher = GraphiQL.createFetcher({{
        url: '{graphql_endpoint_url}',
        method: 'post',
        headers: {{
            'Accept': 'application/json',
            'Content-Type': 'application/json',
        }},
      }});
      const explorerPlugin = GraphiQLPluginExplorer.explorerPlugin();
      root.render(
        React.createElement(GraphiQL, {{
          fetcher,
          query: String.raw`{default_query}`,
          defaultEditorToolsVisibility: false,
          plugins: [explorerPlugin],
        }},
        React.createElement(GraphiQL.Logo, null,
          React.createElement('div', {{ style: {{ display: 'flex', alignItems: 'center' }}}},
            React.createElement('span', null, 'Thoth GraphQL API'),
            React.createElement('img', {{
              src: 'https://cdn.thoth.pub/favicon-96x96.png',
              height: 38,
              alt: 'Thoth Logo',
              style: {{ marginLeft: '10px', marginTop: '-5px' }}
            }}),
          )
        ),
        React.createElement(GraphiQL.Footer, null,
          React.createElement('div', {{ style: {{ display: 'flex', justifyContent: 'space-between' }}}},
            React.createElement('a', {{ href: 'https://thoth.pub/policies/privacy' }}, 'Privacy policy'),
            React.createElement('span', null, 'Thoth Open Metadata'),
          )
        ),
       ),
      );
    </script>
  </body>
</html>

"##,
    )
}
