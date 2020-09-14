use yew::ComponentLink;
use yew::html;
use yew::prelude::*;

use crate::component::dashboard::DashboardComponent;
use crate::component::menu::MenuComponent;
use crate::route::AdminRoute;

pub struct AdminComponent {
    props: Props
}

#[derive(Clone, Properties)]
pub struct Props {
    pub route: AdminRoute,
}

impl Component for AdminComponent {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        AdminComponent { props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
                        <MenuComponent />
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
        }
    }
}
