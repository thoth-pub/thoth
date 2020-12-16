use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::service::cookie::CookieService;
use crate::SESSION_COOKIE;
use crate::THOTH_API;

pub struct NavbarComponent {}

impl Component for NavbarComponent {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        NavbarComponent {}
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> VNode {
        let cookie_service = CookieService::new();
        let authenticated = cookie_service.get(SESSION_COOKIE).is_ok();
        let auth_route = match authenticated {
            true => AppRoute::Home, // will need to handle logout requests here
            false => AppRoute::Login,
        };
        let auth_button = match authenticated {
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
                                <a class="navbar-item" href="https://github.com/OpenBookPublishers/thoth" title="Project">
                                    { "Project" }
                                </a>
                                <a class="navbar-item"  href="https://github.com/orgs/OpenBookPublishers/projects/1" title="Timeline">
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
                            <RouterAnchor<AppRoute>
                                classes="button is-light"
                                route=auth_route
                            >
                                { auth_button }
                            </  RouterAnchor<AppRoute>>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
