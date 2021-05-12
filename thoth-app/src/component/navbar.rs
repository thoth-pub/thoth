use thoth_api::account::model::AccountDetails;
use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::THOTH_GRAPHQL_API;

pub struct NavbarComponent {
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    Logout,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub current_user: Option<AccountDetails>,
    pub callback: Callback<()>,
}

impl Component for NavbarComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        NavbarComponent { props, link }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Logout => {
                self.props.callback.emit(());
                true
            }
        }
    }

    fn view(&self) -> VNode {
        let logout = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Logout
        });
        let graphiql = format!("{}/graphiql", THOTH_GRAPHQL_API);
        html! {
            <nav class="navbar is-warning" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <a class="navbar-item" href="/">
                        <img src="/img/thoth-logo.png" width="50" height="58" style="max-height: none" />
                    </a>

                    <a role="button" class="navbar-burger burger" aria-label="menu" aria-expanded="false" data-target="thothNavbar">
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    </a>
                </div>

                <div id="thothNavbar" class="navbar-menu">
                    <div class="navbar-start">
                        <RouterAnchor<AppRoute>
                            classes="navbar-item"
                            route=AppRoute::Home
                        >
                            {"Catalogue"}
                        </  RouterAnchor<AppRoute>>

                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                            { "Docs" }
                            </a>

                            <div class="navbar-dropdown">
                                <a class="navbar-item" href="https://github.com/thoth-pub/thoth" title="Project">
                                    { "Project" }
                                </a>
                                <a class="navbar-item"  href="https://github.com/thoth-pub/thoth/projects" title="Timeline">
                                    { "Timeline" }
                                </a>
                                <hr class="navbar-divider" />
                                <a class="navbar-item" href={graphiql} title="GraphiQL">
                                    { "GraphiQL" }
                                </a>
                            </div>
                        </div>

                        <RouterAnchor<AppRoute>
                            classes="navbar-item"
                            route=AppRoute::Admin(AdminRoute::Dashboard)
                        >
                            {"Admin"}
                        </  RouterAnchor<AppRoute>>
                    </div>
                </div>

                <div class="navbar-end">
                    <div class="navbar-item">
                        <div class="buttons">
                            <a class="button is-danger" href="https://github.com/thoth-pub/thoth/blob/master/CHANGELOG.md">
                                {"v"}{ env!("CARGO_PKG_VERSION") }
                            </a>
                            {
                                if self.props.current_user.is_some() {
                                    html! {
                                        <button class="button is-light" onclick=logout>
                                            { "Logout" }
                                        </button>
                                    }
                                } else {
                                    html! {
                                        <RouterAnchor<AppRoute> classes="button is-light" route=AppRoute::Login>
                                            {"Login"}
                                        </  RouterAnchor<AppRoute>>
                                    }
                                }
                            }
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
