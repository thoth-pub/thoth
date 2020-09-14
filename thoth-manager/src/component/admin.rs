use yew::ComponentLink;
use yew::html;
use yew::prelude::*;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::dashboard::DashboardComponent;
use crate::component::menu::MenuComponent;
use crate::route::AdminRoute;

pub struct AdminComponent {
    link: ComponentLink<Self>,
    props: Props,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    Clicked,
    ClickedError,
    ClickedWarning,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub route: AdminRoute,
}

impl Component for AdminComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let notification_bus = NotificationBus::dispatcher();

        AdminComponent {
            link,
            props,
            notification_bus,
        }
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
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
                <div class="columns">
                    <div class="column">
                        <div class="container">
                            <MenuComponent route = self.props.route />
                        </div>
                    </div>
                    <div class="column is-four-fifths">
                        <div class="container">
                        {
                            match self.props.route {
                                AdminRoute::Dashboard => html!{<DashboardComponent/>},
                                AdminRoute::Test => html!{{ "TEST" }},
                                AdminRoute::Admin => html!{<DashboardComponent/>},
                            }
                        }
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
