use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;

use crate::agent::session_timer;
use crate::agent::session_timer::SessionTimerAgent;
use crate::agent::session_timer::SessionTimerDispatcher;
use crate::component::contributor::ContributorComponent;
use crate::component::contributors::ContributorsComponent;
use crate::component::dashboard::DashboardComponent;
use crate::component::imprints::ImprintsComponent;
use crate::component::menu::MenuComponent;
use crate::component::new_publisher::NewPublisherComponent;
use crate::component::publications::PublicationsComponent;
use crate::component::publishers::PublishersComponent;
use crate::component::serieses::SeriesesComponent;
use crate::component::work::WorkComponent;
use crate::component::works::WorksComponent;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::service::cookie::CookieService;
use crate::SESSION_COOKIE;

pub struct AdminComponent {
    props: Props,
    _cookie_service: CookieService,
    _link: ComponentLink<Self>,
    _router: RouteAgentDispatcher<()>,
    _session_timer_agent: SessionTimerDispatcher,
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
        let mut router = RouteAgentDispatcher::new();
        let cookie_service = CookieService::new();
        let mut session_timer_agent = SessionTimerAgent::dispatcher();

        if cookie_service.get(SESSION_COOKIE).is_err() {
            router.send(RouteRequest::ChangeRoute(Route::from(AppRoute::Login)));
        } else {
            session_timer_agent.send(session_timer::Request::Start);
        }

        AdminComponent {
            props,
            _cookie_service: cookie_service,
            _link: link,
            _router: router,
            _session_timer_agent: session_timer_agent,
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
                        <MenuComponent route = &self.props.route />
                    </div>
                </div>
                <div class="column is-four-fifths">
                    <div class="container">
                    {
                        match &self.props.route {
                            AdminRoute::Dashboard => html!{<DashboardComponent/>},
                            AdminRoute::Works => html!{<WorksComponent/>},
                            AdminRoute::Work(id) => html!{<WorkComponent work_id = id />},
                            AdminRoute::Publishers => html!{<PublishersComponent/>},
                            AdminRoute::Publisher(id) => html!{{ id }},
                            AdminRoute::NewPublisher => html!{<NewPublisherComponent/>},
                            AdminRoute::Imprints => html!{<ImprintsComponent/>},
                            AdminRoute::Imprint(id) => html!{{ id }},
                            AdminRoute::Publications => html!{<PublicationsComponent/>},
                            AdminRoute::Publication(id) => html!{{ id }},
                            AdminRoute::Contributors => html!{<ContributorsComponent/>},
                            AdminRoute::Contributor(id) => html!{<ContributorComponent contributor_id = id />},
                            AdminRoute::Serieses => html!{<SeriesesComponent/>},
                            AdminRoute::Series(id) => html!{{ id }},
                            AdminRoute::Admin => html!{<DashboardComponent/>},
                        }
                    }
                    </div>
                </div>
            </div>
        }
    }
}
