#[macro_export]
macro_rules! query_builder {
    (
        $request:ident,
        $request_body:ident,
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

        use crate::API_ENDPOINT;

        pub type $fetch = Fetch<$request, $response_body>;
        pub type $fetch_action = FetchAction<$response_body>;

        #[derive(Default, Debug, Clone)]
        pub struct $request {
            pub body: $request_body,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        #[serde(rename_all = "camelCase")]
        pub struct Variables {
            pub work_id: Option<String>,
            pub contributor_id: Option<String>,
            pub limit: Option<i32>,
            pub offset: Option<i32>,
            pub filter: Option<String>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $request_body {
            pub query: String,
            pub variables: Variables,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $response_body {
            pub data: $response_data,
        }

        impl FetchRequest for $request {
            type RequestBody = $request_body;
            type ResponseBody = $response_body;
            type Format = Json;

            fn url(&self) -> String {
                format!("{}/graphql", API_ENDPOINT)
            }

            fn method(&self) -> MethodBody<Self::RequestBody> {
                MethodBody::Post(&self.body)
            }

            fn headers(&self) -> Vec<(String, String)> {
                vec![("Content-Type".to_string(), "application/json".to_string())]
            }

            fn use_cors(&self) -> bool {
                true
            }
        }

        impl Default for $request_body {
            fn default() -> $request_body {
                $request_body {
                    query: $query.to_string(),
                    variables: Variables {
                        work_id: None,
                        contributor_id: None,
                        limit: None,
                        offset: None,
                        filter: None,
                    },
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

pub mod contribution;
pub mod contributor;
pub mod imprint;
pub mod issue;
pub mod publication;
pub mod publisher;
pub mod series;
pub mod stats;
pub mod work;
