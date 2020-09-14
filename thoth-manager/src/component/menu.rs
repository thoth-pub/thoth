use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct MenuComponent {
    props: Props,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub route: AdminRoute,
}

impl MenuComponent {
    fn is_active(&self, route: AdminRoute) -> String {
        if self.props.route == route {
              "is-active".to_string()
          } else {
              "".to_string()
          }
    }
}

impl Component for MenuComponent {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        MenuComponent { props }
    }


    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> VNode {
        html! {
            <aside class="menu">
                <p class="menu-label">
                    { "General" }
                </p>
                <ul class="menu-list">
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Dashboard)}
                            route=AppRoute::Admin(AdminRoute::Dashboard)
                        >
                            {"Dashboard"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Test)}
                            route=AppRoute::Admin(AdminRoute::Test)
                        >
                            {"Test"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                </ul>
            </aside>
        }
    }
}
