//! Utility module to generate a GraphiQL interface

/// Generate the HTML source to show a GraphiQL interface
pub fn graphiql_source(graphql_endpoint_url: &str) -> String {
    let default_query = r#"
# Welcome to Thoth's GraphQL API explorer (GraphiQL).
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
#       works(workType: MONOGRAPH, workStatus: ACTIVE) {
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
    works(order: {field: PUBLICATION_DATE, direction: ASC}) {
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
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Thoth GraphQL API Documentation</title>
        <link rel="shortcut icon" href="https://thoth.pub/favicon.ico" />
        <link rel="stylesheet" type="text/css" href="//cdn.jsdelivr.net/npm/graphiql@0.17.2/graphiql.min.css">
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
        <script src="//cdn.jsdelivr.net/npm/graphiql@0.17.2/graphiql.min.js"></script>
        <script>var GRAPHQL_URL = '{graphql_url}';</script>
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
"#,
        graphql_url = graphql_endpoint_url,
        default_query = default_query,
    )
}
