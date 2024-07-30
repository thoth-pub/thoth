#[macro_export]
macro_rules! graphql_query_builder {
    (
        $request:ident,
        $request_body:ident,
        $variables: ty,
        $query:expr,
        $response_body:ident,
        $response_data:ty,
        $fetch:ident,
        $fetch_action:ident
    ) => {
        use yewtil::fetch::Fetch;
        use yewtil::fetch::FetchAction;
        use yewtil::fetch::FetchRequest;
        use yewtil::fetch::Json;
        use yewtil::fetch::MethodBody;

        use $crate::THOTH_GRAPHQL_API;

        pub type $fetch = Fetch<$request, $response_body>;
        pub type $fetch_action = FetchAction<$response_body>;

        #[derive(Debug, Clone, Default)]
        pub struct $request {
            pub body: $request_body,
        }

        // Some of the variable sets passed in to this macro can derive Eq,
        // but others can't (e.g. ones with members of type f64)
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $request_body {
            pub query: String,
            pub variables: $variables,
        }

        // Some of the data structs passed in to this macro can derive Eq,
        // but others can't (e.g. ones with members of type f64)
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $response_body {
            pub data: $response_data,
        }

        impl FetchRequest for $request {
            type RequestBody = $request_body;
            type ResponseBody = $response_body;
            type Format = Json;

            fn url(&self) -> String {
                format!("{}/graphql", THOTH_GRAPHQL_API)
            }

            fn method(&self) -> MethodBody<Self::RequestBody> {
                MethodBody::Post(&self.body)
            }

            fn headers(&self) -> Vec<(String, String)> {
                use $crate::service::account::AccountService;

                let account_service = AccountService::new();
                let json = ("Content-Type".into(), "application/json".into());
                if let Some(token) = account_service.get_token() {
                    let auth = ("Authorization".into(), format!("Bearer {}", token));
                    vec![json, auth]
                } else {
                    vec![json]
                }
            }

            fn use_cors(&self) -> bool {
                true
            }
        }

        impl Default for $request_body {
            fn default() -> $request_body {
                $request_body {
                    query: $query.to_string(),
                    variables: Default::default(),
                }
            }
        }

        impl Default for $response_body {
            fn default() -> $response_body {
                $response_body {
                    data: Default::default(),
                }
            }
        }
    };
}

use serde::{Deserialize, Serialize};
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use crate::route::AdminRoute;

pub trait Dropdown {
    fn as_dropdown_item(&self, callback: Callback<MouseEvent>) -> Html
    where
        Self: std::fmt::Display,
    {
        // since dropdowns may have an onblur event, we need to use onmousedown instead of
        // onclick. This is not ideal, but it seems to be the only event that'd do the callback
        // without disabling onblur so that onclick can take effect
        html! {
            <div onmousedown={ callback } class="dropdown-item">
                { self }
            </div>
        }
    }
}

pub trait ListString {
    const COMMA_SEPARATOR: &'static str = ", ";

    fn separated_list_item_comma(&self) -> Html {
        self.separated_list_item(false, Self::COMMA_SEPARATOR)
    }

    fn separated_list_item(&self, is_small: bool, separator: &str) -> Html;
}

pub trait CreateRoute {
    fn create_route() -> AdminRoute;
}

pub trait EditRoute {
    fn edit_route(&self) -> AdminRoute;
}

pub trait MetadataTable {
    fn as_table_row(&self, callback: Callback<MouseEvent>) -> Html;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Structure representing a GraphQL type query, in combination with `GraphqlFieldDefinition`, e.g.
///
/// ```graphql
/// {
///   fields: __type(name: "Reference") {
///     fields {
///       name
///       description
///     }
///   }
/// }
pub struct GraphqlFieldList {
    pub fields: Vec<GraphqlFieldDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlFieldDefinition {
    pub name: String,
    pub description: Option<String>,
}

impl GraphqlFieldList {
    /// Get a GraphQL field's description.
    ///
    /// If no description is found, an empty String will be returned.
    pub fn get_description(&self, field_name: &str) -> String {
        match self.fields.iter().find(|f| f.name == field_name) {
            None => Default::default(),
            Some(definition) => definition.description.clone().unwrap_or_default(),
        }
    }
}

pub mod affiliation;
pub mod book;
pub mod chapter;
pub mod contribution;
pub mod contributor;
pub mod funding;
pub mod imprint;
pub mod institution;
pub mod issue;
pub mod language;
pub mod location;
pub mod price;
pub mod publication;
pub mod publisher;
pub mod reference;
pub mod series;
pub mod stats;
pub mod subject;
pub mod work;
pub mod work_relation;
