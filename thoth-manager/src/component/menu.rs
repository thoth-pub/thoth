use crate::route::AppRoute;
use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

pub struct MenuComponent {}

impl Component for MenuComponent {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        MenuComponent {}
    }


    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
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
                        classes="navbar-item"
                        route=AppRoute::Dashboard
                    >
                        {"Dashboard"}
                    </  RouterAnchor<AppRoute>>
                </li>
                <li>
                    <RouterAnchor<AppRoute>
                        classes="navbar-item"
                        route=AppRoute::Test
                    >
                        {"Test"}
                    </  RouterAnchor<AppRoute>>
                </li>
            </ul>
            </aside>
        }
    }
}
