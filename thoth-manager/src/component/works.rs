use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yew_router::route::Route;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;

use crate::api::models::Contribution;
use crate::api::models::Work;
use crate::api::works_query::FetchWorks;
use crate::api::works_query::FetchActionWorks;
use crate::component::utils::Loader;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct WorksComponent {
    markdown: FetchWorks,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionWorks),
    GetMarkdown,
    ChangeRoute(AppRoute),
}

impl Component for WorksComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();

        WorksComponent {
            markdown: Default::default(),
            link,
            router,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.markdown.fetch(Msg::SetMarkdownFetchState));
            self.link
                .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.markdown.apply(fetch_state);
                true
            }
            Msg::GetMarkdown => {
                self.link
                    .send_future(self.markdown.fetch(Msg::SetMarkdownFetchState));
                self.link
                    .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
                false
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r.clone());
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match self.markdown.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {
                    <div class="buttons has-addons is-centered">
                        <button
                            class="button is-success is-large"
                            onclick=self.link.callback(|_| Msg::GetMarkdown)
                        >
                            <span class="icon">
                            <i class="fas fa-sync"></i>
                            </span>
                            <span>{"Reload"}</span>
                        </button>
                    </div>
                }
            }
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => html! {
                <>
                    <nav class="level">
                        <div class="level-left">
                            <div class="level-item">
                                <p class="subtitle is-5">
                                    <strong>{ body.data.works.iter().count() }</strong> { " works" }
                                </p>
                            </div>
                        </div>
                        <div class="level-right">
                            <p class="level-item">
                                <a class="button is-success">{ "New" }</a>
                            </p>
                        </div>
                    </nav>
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "Title" }</th>
                                <th>{ "Contributors" }</th>
                                <th>{ "DOI" }</th>
                                <th>{ "Publisher" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for body.data.works.iter().map(|w| self.render_work(w)) }
                        </tbody>
                    </table>
                </>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}

impl WorksComponent {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }

    fn render_contribution(&self, c: &Contribution) -> Html {
        if c.main_contribution {
            html! {
                <small class="contributor">
                    {&c.contributor.full_name}
                    <span>{ ", " }</span>
                </small>
            }
        } else {
            html! {}
        }
    }

    fn render_work(&self, w: &Work) -> Html {
        html! {
            <tr
                class="row"
                onclick=&self.change_route(AppRoute::Admin(AdminRoute::Work(w.work_id.clone())))
            >
                <td>{&w.work_id}</td>
                <td>{&w.title}</td>
                <td>
                    {
                        if let Some(contributions) = &w.contributions {
                            contributions.iter().map(|c| self.render_contribution(c)).collect::<Html>()
                        } else {
                            html! {}
                        }
                    }
                </td>
                <td>{&w.doi}</td>
                <td>{&w.imprint.publisher.publisher_name}</td>
            </tr>
        }
    }
}
