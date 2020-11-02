use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::prelude::RouterAnchor;
use yew_router::route::Route;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::component::utils::Loader;
use crate::component::utils::Reloader;
use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::Funder;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct FundersComponent {
    get_funders: FetchFunders,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetFundersFetchState(FetchActionFunders),
    GetFunders,
    ChangeRoute(AppRoute),
}

impl Component for FundersComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();

        FundersComponent {
            get_funders: Default::default(),
            link,
            router,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.get_funders.fetch(Msg::SetFundersFetchState));
            self.link
                .send_message(Msg::SetFundersFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFundersFetchState(fetch_state) => {
                self.get_funders.apply(fetch_state);
                true
            }
            Msg::GetFunders => {
                self.link
                    .send_future(self.get_funders.fetch(Msg::SetFundersFetchState));
                self.link
                    .send_message(Msg::SetFundersFetchState(FetchAction::Fetching));
                false
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match self.get_funders.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {<Reloader onclick=self.link.callback(|_| Msg::GetFunders)/>}
            }
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => html! {
                <>
                    <nav class="level">
                        <div class="level-left">
                            <div class="level-item">
                                <p class="subtitle is-5">
                                    <strong>{ body.data.funders.iter().count() }</strong> { " funders" }
                                </p>
                            </div>
                        </div>
                        <div class="level-right">
                            <p class="level-item">
                                <RouterAnchor<AppRoute>
                                    classes="button is-success"
                                    route=AppRoute::Admin(AdminRoute::NewFunder)
                                >
                                    {"New"}
                                </  RouterAnchor<AppRoute>>
                            </p>
                        </div>
                    </nav>
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "Funder" }</th>
                                <th>{ "DOI" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for body.data.funders.iter().map(|p| self.render_funder(p)) }
                        </tbody>
                    </table>
                </>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}

impl FundersComponent {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }

    fn render_funder(&self, f: &Funder) -> Html {
        html! {
            <tr
                class="row"
                onclick=&self.change_route(AppRoute::Admin(AdminRoute::Funder(f.funder_id.clone())))
            >
                <td>{&f.funder_id}</td>
                <td>{&f.funder_name}</td>
                <td>{&f.funder_doi.clone().unwrap_or("".to_string())}</td>
            </tr>
        }
    }
}
