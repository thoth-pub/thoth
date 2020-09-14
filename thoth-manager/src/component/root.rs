use yew::agent::Dispatched;
use yew::agent::Dispatcher;
use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;
use yew_router::route::Route;
use yew_router::switch::Permissive;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::admin::AdminComponent;
use crate::component::catalogue::CatalogueComponent;
use crate::component::login::LoginComponent;
use crate::component::navbar::NavbarComponent;
use crate::component::notification::NotificationComponent;
use crate::route::AppRoute;

pub struct RootComponent {
    link: ComponentLink<RootComponent>,
    notification_bus: Dispatcher<NotificationBus>,
}

pub enum Msg {
    Clicked,
    ClickedError,
    ClickedWarning,
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let notification_bus = NotificationBus::dispatcher();

        RootComponent {
            notification_bus,
            link,
        }
    }


    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                self.notification_bus
                    .send(Request::NotificationBusMsg(
                            ("All good".to_string(), NotificationStatus::Success)));
                false
            }
            Msg::ClickedError => {
                self.notification_bus
                    .send(Request::NotificationBusMsg(
                            ("Something terrible happened".to_string(), NotificationStatus::Danger)));
                false
            }
            Msg::ClickedWarning => {
                self.notification_bus
                    .send(Request::NotificationBusMsg(
                            ("This is a warning".to_string(), NotificationStatus::Warning)));
                false
            }
        }
    }

    fn view(&self) -> VNode {
        html! {
            <>
                <header>
                    <NavbarComponent />
                </header>
                <NotificationComponent />
                <div class="main section">
                <div class="buttons">
                    <button
                        class="button"
                        onclick=self.link.callback(|_| Msg::Clicked)
                    >
                        {"Notify"}
                    </button>
                    <button
                        class="button"
                        onclick=self.link.callback(|_| Msg::ClickedError)
                    >
                        {"Notify"}
                    </button>
                    <button
                        class="button"
                        onclick=self.link.callback(|_| Msg::ClickedWarning)
                    >
                        {"Notify"}
                    </button>
                </div>

                <Router<AppRoute>
                    render = Router::render(|switch: AppRoute| {
                        match switch {
                            AppRoute::Home => {
                                html! {
                                    <>
                                        <CatalogueComponent />
                                    </>
                                }
                            }
                            AppRoute::Login => {
                                html! {
                                    <LoginComponent />
                                }
                            }
                            AppRoute::Admin(admin_route) => {
                                html! {
                                    <AdminComponent route = admin_route />
                                }

                            }
                            AppRoute::Loading => {
                                html! {
                                    <div class="uk-position-center", uk-icon="icon: cloud-download; ratio: 3",  ></div>
                                }
                            }
                            AppRoute::Error(Permissive(None)) => {
                                html! {
                                    <div class="uk-position-center", uk-icon="icon: ban; ratio: 3",></div>
                                }
                            }
                            AppRoute::Error(Permissive(Some(missed_route))) => {
                                html!{
                                    format!("Page '{}' not found", missed_route)
                                }
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
