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

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        MenuComponent { props }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
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
                </ul>
                <p class="menu-label">
                    { "Metadata" }
                </p>
                <ul class="menu-list">
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Works)}
                            route=AppRoute::Admin(AdminRoute::Works)
                        >
                            {"Works"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <ul class="menu-list">
                            <li>
                                <RouterAnchor<AppRoute>
                                    classes={self.is_active(AdminRoute::Books)}
                                    route=AppRoute::Admin(AdminRoute::Books)
                                >
                                    {"Books"}
                                </  RouterAnchor<AppRoute>>
                            </li>
                            <li>
                                <RouterAnchor<AppRoute>
                                    classes={self.is_active(AdminRoute::Chapters)}
                                    route=AppRoute::Admin(AdminRoute::Chapters)
                                >
                                    {"Chapters"}
                                </  RouterAnchor<AppRoute>>
                            </li>
                        </ul>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Publications)}
                            route=AppRoute::Admin(AdminRoute::Publications)
                        >
                            {"Publications"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Publishers)}
                            route=AppRoute::Admin(AdminRoute::Publishers)
                        >
                            {"Publishers"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Imprints)}
                            route=AppRoute::Admin(AdminRoute::Imprints)
                        >
                            {"Imprints"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Contributors)}
                            route=AppRoute::Admin(AdminRoute::Contributors)
                        >
                            {"Contributors"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Serieses)}
                            route=AppRoute::Admin(AdminRoute::Serieses)
                        >
                            {"Series"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Funders)}
                            route=AppRoute::Admin(AdminRoute::Funders)
                        >
                            {"Funders"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                </ul>

            </aside>
        }
    }
}
