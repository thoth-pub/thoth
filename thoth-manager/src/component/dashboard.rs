use yew::ComponentLink;
use yew::html;
use yew::prelude::*;
use yew_router::prelude::RouterAnchor;

use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct DashboardComponent {}

impl Component for DashboardComponent {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DashboardComponent{}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
        <div class="tile is-ancestor">
            <div class="tile is-parent">
                <article class="tile is-child notification is-primary">
                    <div class="content">
                        <p class="title">{"238 Works"}</p>
                        <RouterAnchor<AppRoute>
                            route=AppRoute::Admin(AdminRoute::Works)
                        >
                            {"See all"}
                        </  RouterAnchor<AppRoute>>
                    </div>
                </article>
            </div>
            <div class="tile is-parent">
                <article class="tile is-child notification is-link">
                    <div class="content">
                        <p class="title">{"2 Publishers"}</p>
                        <RouterAnchor<AppRoute>
                            route=AppRoute::Admin(AdminRoute::Publishers)
                        >
                            {"See all"}
                        </  RouterAnchor<AppRoute>>
                    </div>
                </article>
            </div>
        </div>
        }
    }
}
