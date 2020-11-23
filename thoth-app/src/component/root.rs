use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;
use yew_router::route::Route;
use yew_router::switch::Permissive;

use crate::component::admin::AdminComponent;
use crate::component::catalogue::CatalogueComponent;
use crate::component::hero::HeroComponent;
use crate::component::login::LoginComponent;
use crate::component::navbar::NavbarComponent;
use crate::component::notification::NotificationComponent;
use crate::route::AppRoute;

pub struct RootComponent {}

impl Component for RootComponent {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        RootComponent {}
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> VNode {
        html! {
            <>
                <header>
                    <NavbarComponent />
                </header>
                <NotificationComponent />
                <div class="main">
                    <Router<AppRoute>
                        render = Router::render(|switch: AppRoute| {
                            match switch {
                                AppRoute::Home => html! {
                                    <>
                                        <HeroComponent />
                                        <div class="section">
                                            <CatalogueComponent />
                                        </div>
                                    </>
                                },
                                AppRoute::Login => html! {
                                    <div class="section">
                                        <LoginComponent />
                                    </div>
                                },
                                AppRoute::Admin(admin_route) => html! {
                                    <div class="section">
                                        <AdminComponent route={admin_route} />
                                    </div>
                                },
                                AppRoute::Error(Permissive(None)) => html! {
                                    <div class="uk-position-center"></div>
                                },
                                AppRoute::Error(Permissive(Some(missed_route))) => html!{
                                    format!("Page '{}' not found", missed_route)
                                }
                            }
                        })
                        redirect = Router::redirect(|route: Route| {
                            AppRoute::Error(Permissive(Some(route.route)))
                        })
                    />
                </div>
            </>
        }
    }
}
