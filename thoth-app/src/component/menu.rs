use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

use crate::route::AdminRoute;

pub struct MenuComponent {}

#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub route: AdminRoute,
}

impl MenuComponent {
    fn is_active(&self, route: AdminRoute, ctx: &Context<Self>) -> Classes {
        // This relies on the history listener in admin.rs triggering a props update
        // on route change; changes of route do not otherwise re-render this component
        if ctx.props().route == route {
            "is-active".into()
        } else {
            "".into()
        }
    }
}

impl Component for MenuComponent {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        MenuComponent {}
    }

    fn view(&self, ctx: &Context<Self>) -> VNode {
        html! {
            <aside class="menu">
                <p class="menu-label">
                    { "General" }
                </p>
                <ul class="menu-list">
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Dashboard, ctx)}
                            to={ AdminRoute::Dashboard }
                        >
                            {"Dashboard"}
                        </Link<AdminRoute>>
                    </li>
                </ul>
                <p class="menu-label">
                    { "Metadata" }
                </p>
                <ul class="menu-list">
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Works, ctx)}
                            to={ AdminRoute::Works }
                        >
                            {"Works"}
                        </Link<AdminRoute>>
                    </li>
                    <li>
                        <ul class="menu-list">
                            <li>
                                <Link<AdminRoute>
                                    classes={self.is_active(AdminRoute::Books, ctx)}
                                    to={ AdminRoute::Books }
                                >
                                    {"Books"}
                                </Link<AdminRoute>>
                            </li>
                            <li>
                                <Link<AdminRoute>
                                    classes={self.is_active(AdminRoute::Chapters, ctx)}
                                    to={ AdminRoute::Chapters }
                                >
                                    {"Chapters"}
                                </Link<AdminRoute>>
                            </li>
                        </ul>
                    </li>
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Publications, ctx)}
                            to={ AdminRoute::Publications }
                        >
                            {"Publications"}
                        </Link<AdminRoute>>
                    </li>
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Publishers, ctx)}
                            to={ AdminRoute::Publishers }
                        >
                            {"Publishers"}
                        </Link<AdminRoute>>
                    </li>
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Imprints, ctx)}
                            to={ AdminRoute::Imprints }
                        >
                            {"Imprints"}
                        </Link<AdminRoute>>
                    </li>
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Contributors, ctx)}
                            to={ AdminRoute::Contributors }
                        >
                            {"Contributors"}
                        </Link<AdminRoute>>
                    </li>
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Serieses, ctx)}
                            to={ AdminRoute::Serieses }
                        >
                            {"Series"}
                        </Link<AdminRoute>>
                    </li>
                    <li>
                        <Link<AdminRoute>
                            classes={self.is_active(AdminRoute::Institutions, ctx)}
                            to={ AdminRoute::Institutions }
                        >
                            {"Institutions"}
                        </Link<AdminRoute>>
                    </li>
                </ul>

            </aside>
        }
    }
}
