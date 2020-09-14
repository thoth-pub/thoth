use yew::ComponentLink;
use yew::html;
use yew::prelude::*;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::component::dashboard::DashboardComponent;
use crate::component::menu::MenuComponent;
use crate::route::AdminRoute;

pub struct AdminComponent {
    link: ComponentLink<Self>,
    props: Props,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {}

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

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
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
                            AdminRoute::Works => html!{{ "Works" }},
                            AdminRoute::Admin => html!{<DashboardComponent/>},
                        }
                    }
                    </div>
                </div>
            </div>
        }
    }
}
