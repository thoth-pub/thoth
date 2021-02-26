#[macro_export]
macro_rules! pagination_helpers {
    ($component:ident, $pagination_text:ident, $search_text:ident) => {
        use crate::string::$pagination_text;
        use crate::string::$search_text;

        impl $component {
            fn search_text(&self) -> String {
                format!("{}", $search_text)
            }

            fn display_count(&self) -> String {
                let offset_display = match self.offset == 0 && self.result_count > 0 {
                    true => 1,
                    false => self.offset,
                };
                let limit_display = match self.limit > self.result_count {
                    true => self.result_count,
                    false => self.limit + self.offset,
                };
                format!("{} {}-{} of {}", $pagination_text, offset_display, limit_display, self.result_count)
            }

            fn is_previous_disabled(&self) -> bool {
                self.offset < self.page_size
            }

            fn is_next_disabled(&self) -> bool {
                self.limit >= self.result_count
            }

            #[allow(dead_code)]
            fn pagination_controls(&self) -> Html {
                html! {
                    <nav class="pagination is-centered" role="navigation" aria-label="pagination">
                        <a class="pagination-previous"
                            onclick=self.link.callback(|_| Msg::PreviousPage)
                            disabled=self.is_previous_disabled()
                        >{ crate::string::PREVIOUS_PAGE_BUTTON }</a>
                        <a class="pagination-next"
                            onclick=self.link.callback(|_| Msg::NextPage)
                            disabled=self.is_next_disabled()
                        >{ crate::string::NEXT_PAGE_BUTTON }</a>
                        <div class="pagination-list">
                            <div class="field" style="width: 80%">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        value=self.search_term
                                        placeholder=self.search_text()
                                        oninput=self.link.callback(|e: InputData| Msg::Search(e.value))
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                            </div>
                        </div>
                    </nav>
                }
            }
        }
    }
}

#[macro_export]
macro_rules! pagination_component {
    (
        $component:ident,
        $entity:ty,
        $result:ident,
        $result_count:ident,
        $request:ident,
        $fetch_action:ty,
        $fetch_data:ty,
        $request_body:ident,
        $request_variables:ident,
        $search_text:ident,
        $pagination_text:ident,
        $table_headers:expr
    ) => {
        use thoth_api::account::model::AccountDetails;
        use yew::html;
        use yew::prelude::Component;
        use yew::prelude::Html;
        use yew::prelude::InputData;
        use yew::prelude::Properties;
        use yew::prelude::ShouldRender;
        use yew::ComponentLink;
        use yew_router::agent::RouteAgentDispatcher;
        use yew_router::agent::RouteRequest;
        use yew_router::prelude::RouterAnchor;
        use yew_router::route::Route;
        use yewtil::fetch::Fetch;
        use yewtil::fetch::FetchAction;
        use yewtil::fetch::FetchState;
        use yewtil::future::LinkFuture;

        use crate::component::utils::Loader;
        use crate::component::utils::Reloader;
        use crate::route::AppRoute;

        pub struct $component {
            limit: i32,
            offset: i32,
            page_size: i32,
            search_term: String,
            data: Vec<$entity>,
            table_headers: Vec<String>,
            result_count: i32,
            fetch_data: $fetch_data,
            link: ComponentLink<Self>,
            router: RouteAgentDispatcher<()>,
            props: Props,
        }

        pagination_helpers! {$component, $pagination_text, $search_text}

        pub enum Msg {
            SetFetchState($fetch_action),
            GetData,
            PaginateData,
            Search(String),
            NextPage,
            PreviousPage,
            ChangeRoute(AppRoute),
        }

        #[derive(Clone, Properties)]
        pub struct Props {
            pub current_user: AccountDetails,
        }

        impl Component for $component {
            type Message = Msg;
            type Properties = Props;

            fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
                let router = RouteAgentDispatcher::new();
                let offset: i32 = Default::default();
                let page_size: i32 = 20;
                let limit: i32 = page_size;
                let search_term: String = Default::default();
                let result_count: i32 = Default::default();
                let data = Default::default();
                let fetch_data = Default::default();
                let table_headers = $table_headers;

                link.send_message(Msg::PaginateData);

                $component {
                    limit,
                    offset,
                    page_size,
                    search_term,
                    data,
                    table_headers,
                    result_count,
                    fetch_data,
                    link,
                    router,
                    props,
                }
            }

            fn update(&mut self, msg: Self::Message) -> ShouldRender {
                match msg {
                    Msg::SetFetchState(fetch_state) => {
                        self.fetch_data.apply(fetch_state);
                        self.data = match self.fetch_data.as_ref().state() {
                            FetchState::Fetched(body) => body.data.$result.clone(),
                            _ => Default::default(),
                        };
                        self.result_count = match self.fetch_data.as_ref().state() {
                            FetchState::Fetched(body) => body.data.$result_count,
                            _ => Default::default(),
                        };
                        true
                    }
                    Msg::GetData => {
                        self.link
                            .send_future(self.fetch_data.fetch(Msg::SetFetchState));
                        self.link
                            .send_message(Msg::SetFetchState(FetchAction::Fetching));
                        false
                    }
                    Msg::PaginateData => {
                        let filter = self.search_term.clone();
                        let body = $request_body {
                            variables: $request_variables {
                                limit: Some(self.limit),
                                offset: Some(self.offset),
                                filter: Some(filter),
                                publishers: self.props.current_user.resource_access.restricted_to(),
                            },
                            ..Default::default()
                        };
                        let request = $request { body };
                        self.fetch_data = Fetch::new(request);
                        self.link.send_message(Msg::GetData);
                        false
                    }
                    Msg::Search(term) => {
                        self.limit = self.page_size;
                        self.offset = 0;
                        self.search_term = term;
                        self.link.send_message(Msg::PaginateData);
                        false
                    }
                    Msg::NextPage => {
                        if self.limit < self.result_count && !self.is_next_disabled() {
                            self.offset += self.page_size;
                            self.link.send_message(Msg::PaginateData);
                        }
                        false
                    }
                    Msg::PreviousPage => {
                        if self.offset > 0 && !self.is_previous_disabled() {
                            self.offset -= self.page_size;
                            self.link.send_message(Msg::PaginateData);
                        }
                        false
                    }
                    Msg::ChangeRoute(r) => {
                        let route = Route::from(r);
                        self.router.send(RouteRequest::ChangeRoute(route));
                        false
                    }
                }
            }

            fn change(&mut self, props: Self::Properties) -> ShouldRender {
                self.props = props;
                true
            }

            fn view(&self) -> Html {
                let route = <$entity>::create_route();
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="level-item">
                                    <span>
                                    { self.display_count() }
                                    </span>
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                        <RouterAnchor<AppRoute>
                                            classes="button is-success"
                                            route={route}
                                        >
                                            {"New"}
                                        </  RouterAnchor<AppRoute>>
                                </p>
                            </div>
                        </nav>
                        { self.pagination_controls() }
                        {
                            match self.fetch_data.as_ref().state() {
                                FetchState::NotFetching(_) => {
                                    html! {<Reloader onclick=self.link.callback(|_| Msg::GetData)/>}
                                },
                                FetchState::Fetching(_) => html! {<Loader/>},
                                FetchState::Fetched(_body) => html! {
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr>
                                                {
                                                    for self.table_headers.iter().map(|h| {
                                                        html! {<th>{h}</th>}
                                                    })
                                                }
                                            </tr>
                                        </thead>

                                        <tbody>
                                            {
                                                for self.data.iter().map(|r| {
                                                    let route = r.edit_route().clone();
                                                    r.as_table_row(
                                                        self.link.callback(move |_| {
                                                            Msg::ChangeRoute(route.clone())
                                                        })
                                                    )
                                                })
                                            }
                                        </tbody>
                                    </table>
                                },
                                FetchState::Failed(_, err) => html! {&err},
                            }
                        }
                    </>
                }
            }
        }
    };
}

pub mod admin;
pub mod catalogue;
pub mod contributions_form;
pub mod contributor;
pub mod contributors;
pub mod dashboard;
pub mod funder;
pub mod funders;
pub mod fundings_form;
pub mod hero;
pub mod imprint;
pub mod imprints;
pub mod issues_form;
pub mod languages_form;
pub mod login;
pub mod menu;
pub mod navbar;
pub mod new_contributor;
pub mod new_funder;
pub mod new_imprint;
pub mod new_publisher;
pub mod new_series;
pub mod new_work;
pub mod notification;
pub mod prices_form;
pub mod publication;
pub mod publications;
pub mod publications_form;
pub mod publisher;
pub mod publishers;
pub mod root;
pub mod series;
pub mod serieses;
pub mod subjects_form;
pub mod utils;
pub mod work;
pub mod works;
