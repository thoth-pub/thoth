use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;
use thoth_api::account::model::AccountDetails;

use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::service::account::AccountService;
use crate::THOTH_API;

pub struct NavbarComponent {
    props: Props,
    link: ComponentLink<Self>,
    account_service: AccountService,
}

pub enum Msg {
    Login,
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
        let account_service = AccountService::new();
        NavbarComponent {
            props,
            link,
            account_service,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Login => {
                self.account_service.redirect_to_login();
                true
            }
            Msg::Logout => {
                self.props.callback.emit(());
                true
            }
        }
    }

    fn view(&self) -> VNode {
        let auth_action = match &self.props.current_user.is_some() {
            true => self.link.callback(|e: MouseEvent| {
                e.prevent_default();
                Msg::Logout
            }),
            false => self.link.callback(|e: MouseEvent| {
                e.prevent_default();
                Msg::Login
            }),
        };
        let auth_button = match &self.props.current_user.is_some() {
            true => "Logout",
            false => "Log in",
        };
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
                                <a class="navbar-item" href={format!("{}/graphiql", THOTH_API)} title="GraphiQL">
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
                            <button
                                class="button is-light"
                                onclick=auth_action
                            >
                                { auth_button }
                            </button>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
