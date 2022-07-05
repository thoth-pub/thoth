use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;
use yew_router::scope_ext::HistoryHandle;

use crate::route::AdminRoute;

pub struct MenuComponent {
    _listener: Option<HistoryHandle>,
}

pub enum Msg {
    RouteChanged,
}

impl MenuComponent {
    fn is_active(&self, route: AdminRoute, ctx: &Context<Self>) -> Classes {
        if ctx.link().route::<AdminRoute>() == Some(route) {
            "is-active".into()
        } else {
            "".into()
        }
    }
}

impl Component for MenuComponent {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Listen for when the route changes, e.g. when user clicks on a menu item
        let listener = ctx
            .link()
            .add_history_listener(ctx.link().callback(move |_| Msg::RouteChanged));
        MenuComponent {
            _listener: listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RouteChanged => {
                // Trigger a re-render to update menu items' "is-active" classes
                true
            }
        }
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
