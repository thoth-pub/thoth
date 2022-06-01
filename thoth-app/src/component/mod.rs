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
                let limit_display = match (self.limit + self.offset) > self.result_count {
                    true => self.result_count,
                    false => self.limit + self.offset,
                };
                format!("{} {}-{} of {}", $pagination_text, offset_display, limit_display, self.result_count)
            }

            fn is_previous_disabled(&self) -> bool {
                self.offset < self.page_size
            }

            fn is_next_disabled(&self) -> bool {
                self.limit + self.offset >= self.result_count
            }

            #[allow(dead_code)]
            fn pagination_controls(&self, ctx: &Context<Self>) -> Html {
                html! {
                    <nav class="pagination is-centered" role="navigation" aria-label="pagination">
                        <a class="pagination-previous"
                            onclick={ ctx.link().callback(|_| Msg::PreviousPage) }
                            disabled={ self.is_previous_disabled() }
                        >{ crate::string::PREVIOUS_PAGE_BUTTON }</a>
                        <a class="pagination-next"
                            onclick={ ctx.link().callback(|_| Msg::NextPage) }
                            disabled={ self.is_next_disabled() }
                        >{ crate::string::NEXT_PAGE_BUTTON }</a>
                        <div class="pagination-list">
                            <div class="field" style="width: 80%">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        value={ self.search_term.clone() }
                                        placeholder={ self.search_text() }
                                        oninput={ ctx.link().callback(|e: InputData| Msg::Search(e.value)) }
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
        $table_headers:expr,
        $order_struct:ty,
        $order_field:ty,
    ) => {
        use std::str::FromStr;
        use thoth_api::account::model::AccountAccess;
        use thoth_api::account::model::AccountDetails;
        use thoth_api::graphql::utils::Direction::*;
        use yew::html;
        use yew::prelude::Component;
        use yew::prelude::Context;
        use yew::prelude::Html;
        use yew::prelude::InputData;
        use yew::prelude::Properties;
        use yew_router::agent::RouteAgentDispatcher;
        use yew_router::agent::RouteRequest;
        use yew_router::prelude::RouterAnchor;
        use yew_router::route::Route;
        use yewtil::fetch::Fetch;
        use yewtil::fetch::FetchAction;
        use yewtil::fetch::FetchState;
        use yewtil::future::LinkFuture;
        use yewtil::NeqAssign;

        use crate::component::utils::Loader;
        use crate::component::utils::Reloader;
        use crate::models::{EditRoute, CreateRoute, MetadataTable};
        use crate::route::AppRoute;

        pub struct $component {
            limit: i32,
            offset: i32,
            page_size: i32,
            search_term: String,
            order: $order_struct,
            data: Vec<$entity>,
            table_headers: Vec<String>,
            result_count: i32,
            fetch_data: $fetch_data,
            router: RouteAgentDispatcher<()>,
            // Store props value locally in order to test whether it has been updated on props change
            resource_access: AccountAccess,
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
            SortColumn($order_field),
        }

        #[derive(Clone, Properties)]
        pub struct Props {
            pub current_user: AccountDetails,
        }

        impl Component for $component {
            type Message = Msg;
            type Properties = Props;

            fn create(ctx: &Context<Self>) -> Self {
                let router = RouteAgentDispatcher::new();
                let offset: i32 = Default::default();
                let page_size: i32 = 20;
                let limit: i32 = page_size;
                let search_term: String = Default::default();
                let order = Default::default();
                let result_count: i32 = Default::default();
                let data = Default::default();
                let fetch_data = Default::default();
                let table_headers = $table_headers;
                let resource_access = ctx.props().current_user.resource_access;

                ctx.link().send_message(Msg::PaginateData);

                $component {
                    limit,
                    offset,
                    page_size,
                    search_term,
                    order,
                    data,
                    table_headers,
                    result_count,
                    fetch_data,
                    router,
                    resource_access,
                }
            }

            fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                        ctx.link()
                            .send_future(self.fetch_data.fetch(Msg::SetFetchState));
                        ctx.link()
                            .send_message(Msg::SetFetchState(FetchAction::Fetching));
                        false
                    }
                    Msg::PaginateData => {
                        let filter = self.search_term.clone();
                        let order = self.order.clone();
                        let body = $request_body {
                            variables: $request_variables {
                                limit: Some(self.limit),
                                offset: Some(self.offset),
                                filter: Some(filter),
                                order: Some(order),
                                publishers: self.resource_access.restricted_to(),
                            },
                            ..Default::default()
                        };
                        let request = $request { body };
                        self.fetch_data = Fetch::new(request);
                        ctx.link().send_message(Msg::GetData);
                        false
                    }
                    Msg::Search(term) => {
                        self.offset = 0;
                        self.search_term = term;
                        ctx.link().send_message(Msg::PaginateData);
                        false
                    }
                    Msg::NextPage => {
                        if self.limit < self.result_count && !self.is_next_disabled() {
                            self.offset += self.page_size;
                            ctx.link().send_message(Msg::PaginateData);
                        }
                        false
                    }
                    Msg::PreviousPage => {
                        if self.offset > 0 && !self.is_previous_disabled() {
                            self.offset -= self.page_size;
                            ctx.link().send_message(Msg::PaginateData);
                        }
                        false
                    }
                    Msg::ChangeRoute(r) => {
                        let route = Route::from(r);
                        self.router.send(RouteRequest::ChangeRoute(route));
                        false
                    }
                    Msg::SortColumn(header) => {
                        // Clicking on a header, if enabled, sorts the table by that column ascending
                        // Clicking on the current sort column header reverses the sort direction
                        self.order.direction = match self.order.field.neq_assign(header) {
                            true => Asc,
                            false => match self.order.direction {
                                Asc => Desc,
                                Desc => Asc,
                            },
                        };
                        self.offset = 0;
                        ctx.link().send_message(Msg::PaginateData);
                        false
                    }
                }
            }

            fn changed(&mut self, ctx: &Context<Self>) -> bool {
                let updated_permissions =
                    self.resource_access.neq_assign(ctx.props().current_user.resource_access);
                if updated_permissions {
                    ctx.link().send_message(Msg::PaginateData);
                }
                false
            }

            fn view(&self, ctx: &Context<Self>) -> Html {
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
                        { self.pagination_controls(ctx) }
                        {
                            match self.fetch_data.as_ref().state() {
                                FetchState::NotFetching(_) => {
                                    html! {<Reloader onclick={ ctx.link().callback(|_| Msg::GetData) }/>}
                                },
                                FetchState::Fetching(_) => html! {<Loader/>},
                                FetchState::Fetched(_body) => html! {
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr>
                                                {
                                                    for self.table_headers.iter().map(|h| {
                                                        {
                                                            // If the header is a sortable field, make it clickable
                                                            match <$order_field>::from_str(&h) {
                                                                Ok(header) => {
                                                                    html! {
                                                                        <th class="th is-clickable"
                                                                            onclick={ ctx.link().callback(move |_| {
                                                                                Msg::SortColumn(header.clone())
                                                                            }) }
                                                                        >
                                                                            {h}
                                                                        </th>
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    html! {<th>{h}</th>}
                                                                }
                                                            }
                                                        }
                                                    })
                                                }
                                            </tr>
                                        </thead>

                                        <tbody>
                                            {
                                                for self.data.iter().map(|r| {
                                                    let route = r.edit_route().clone();
                                                    r.as_table_row(
                                                        ctx.link().callback(move |_| {
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

pub trait ToOption {
    fn to_opt_string(self) -> Option<String>;
    fn to_opt_float(self) -> Option<f64>;
    fn to_opt_int(self) -> Option<i32>;
}

impl ToOption for String {
    fn to_opt_string(self) -> Option<String> {
        match self.trim().is_empty() {
            true => None,
            false => Some(self.trim().to_owned()),
        }
    }

    fn to_opt_float(self) -> Option<f64> {
        let value = self.parse().unwrap_or(0.0);
        match value == 0.0 {
            true => None,
            false => Some(value),
        }
    }

    fn to_opt_int(self) -> Option<i32> {
        let value = self.parse().unwrap_or(0);
        match value == 0 {
            true => None,
            false => Some(value),
        }
    }
}

pub trait ToElementValue {
    fn to_value(self) -> String;
}

impl ToElementValue for yew::InputEvent {
    fn to_value(self) -> String {
        use wasm_bindgen::JsCast;
        use web_sys::HtmlInputElement;
        self.target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            .map(|i| i.value())
            .unwrap_or_default()
    }
}

pub mod admin;
pub mod affiliations_form;
pub mod books;
pub mod catalogue;
pub mod chapters;
pub mod contributions_form;
pub mod contributor;
pub mod contributors;
pub mod dashboard;
pub mod delete_dialogue;
pub mod fundings_form;
pub mod hero;
pub mod imprint;
pub mod imprints;
pub mod institution;
pub mod institutions;
pub mod issues_form;
pub mod languages_form;
pub mod locations_form;
pub mod login;
pub mod menu;
pub mod navbar;
pub mod new_contributor;
pub mod new_imprint;
pub mod new_institution;
pub mod new_publisher;
pub mod new_series;
pub mod new_work;
pub mod notification;
pub mod prices_form;
pub mod publication;
pub mod publication_modal;
pub mod publications;
pub mod publications_form;
pub mod publisher;
pub mod publishers;
pub mod related_works_form;
pub mod root;
pub mod series;
pub mod serieses;
pub mod subjects_form;
pub mod utils;
pub mod work;
pub mod works;
