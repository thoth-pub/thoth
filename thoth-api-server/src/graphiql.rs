//! Utility module to generate a GraphiQL interface

/// Generate the HTML source to show a GraphiQL interface
pub fn graphiql_source(graphql_endpoint_url: &str) -> String {
    let default_query = r#"# Welcome to Thoth's GraphQL API explorer (GraphiQL).
#
# GraphiQL is an in-browser tool for writing, validating, and
# testing GraphQL queries.
#
# Type queries into this side of the screen, and you will see intelligent
# typeaheads aware of the current GraphQL type schema and live syntax and
# validation errors highlighted within the text.
#
# Click on the QueryRoot in the Documentation Explorer ( < Docs ) on the
# right hand side of the screen to navigate the API schema.
#
# GraphQL queries typically start with a "{" character. Lines that starts
# with a # are ignored.
#
# An example Thoth GraphQL query might look like:
#
#     {
#       books(workStatuses: [ACTIVE]) {
#           fullTitle
#           doi
#       }
#     }
#
# Keyboard shortcuts:
#
#       Run Query:  Ctrl-Enter (or press the play button above)
#
#   Auto Complete:  Ctrl-Space (or just start typing)
#
#
# Run the following query (Ctrl-Enter):
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
"#;

    format!(
        r##"
<!DOCTYPE html>
<html>
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
        <link rel="manifest" href="https://cdn.thoth.pub/manifest.json">
        <meta name="msapplication-TileColor" content="#FFDD57">
        <meta name="msapplication-TileImage" content="https://cdn.thoth.pub/ms-icon-144x144.png">
        <meta name="theme-color" content="#FFDD57">
        <link rel="stylesheet" type="text/css" href="//cdn.jsdelivr.net/npm/graphiql@0.17.5/graphiql.min.css">
          <style>
            html, body, #app {{
                height: 100vh;
                margin: 0;
                overflow: hidden;
            }}
        </style>
    </head>
    <body>
        <div id="app"></div>
        <script src="//cdnjs.cloudflare.com/ajax/libs/fetch/2.0.3/fetch.js"></script>
        <script src="//cdnjs.cloudflare.com/ajax/libs/react/16.10.2/umd/react.production.min.js"></script>
        <script src="//cdnjs.cloudflare.com/ajax/libs/react-dom/16.10.2/umd/react-dom.production.min.js"></script>
        <script src="//cdn.jsdelivr.net/npm/graphiql@0.17.5/graphiql.min.js"></script>
        <script>var GRAPHQL_URL = '{graphql_endpoint_url}';</script>
        <script>
            function graphQLFetcher(params) {{
                return fetch(GRAPHQL_URL, {{
                    method: 'post',
                    headers: {{
                        'Accept': 'application/json',
                        'Content-Type': 'application/json',
                    }},
                    credentials: 'include',
                    body: JSON.stringify(params)
                }}).then(function (response) {{
                    return response.text();
                }}).then(function (body) {{
                    try {{
                        return JSON.parse(body);
                    }} catch (error) {{
                        return body;
                    }}
                }});
            }}
            ReactDOM.render(
                React.createElement(GraphiQL, {{
                    fetcher: graphQLFetcher,
                    query: String.raw`{default_query}`,
                }}),
                document.querySelector('#app'));
        </script>
    </body>
</html>
"##,
    )
}
