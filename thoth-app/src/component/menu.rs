use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct MenuComponent {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub route: AdminRoute,
}

impl MenuComponent {
    fn is_active(&self, route: AdminRoute, ctx: &Context<Self>) -> String {
        if ctx.props().route == route {
            "is-active".to_string()
        } else {
            "".to_string()
        }
    }
}

impl Component for MenuComponent {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        MenuComponent {}
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> VNode {
        html! {
            <aside class="menu">
                <p class="menu-label">
                    { "General" }
                </p>
                <ul class="menu-list">
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Dashboard, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Dashboard) }
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
                            classes={self.is_active(AdminRoute::Works, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Works) }
                        >
                            {"Works"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <ul class="menu-list">
                            <li>
                                <RouterAnchor<AppRoute>
                                    classes={self.is_active(AdminRoute::Books, ctx)}
                                    route={ AppRoute::Admin(AdminRoute::Books) }
                                >
                                    {"Books"}
                                </  RouterAnchor<AppRoute>>
                            </li>
                            <li>
                                <RouterAnchor<AppRoute>
                                    classes={self.is_active(AdminRoute::Chapters, ctx)}
                                    route={ AppRoute::Admin(AdminRoute::Chapters) }
                                >
                                    {"Chapters"}
                                </  RouterAnchor<AppRoute>>
                            </li>
                        </ul>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Publications, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Publications) }
                        >
                            {"Publications"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Publishers, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Publishers) }
                        >
                            {"Publishers"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Imprints, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Imprints) }
                        >
                            {"Imprints"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Contributors, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Contributors) }
                        >
                            {"Contributors"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Serieses, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Serieses) }
                        >
                            {"Series"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                    <li>
                        <RouterAnchor<AppRoute>
                            classes={self.is_active(AdminRoute::Institutions, ctx)}
                            route={ AppRoute::Admin(AdminRoute::Institutions) }
                        >
                            {"Institutions"}
                        </  RouterAnchor<AppRoute>>
                    </li>
                </ul>

            </aside>
        }
    }
}
