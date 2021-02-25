use thoth_api::account::model::AccountDetails;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;

use crate::component::contributor::ContributorComponent;
use crate::component::contributors::ContributorsComponent;
use crate::component::dashboard::DashboardComponent;
use crate::component::funder::FunderComponent;
use crate::component::funders::FundersComponent;
use crate::component::imprint::ImprintComponent;
use crate::component::imprints::ImprintsComponent;
use crate::component::menu::MenuComponent;
use crate::component::new_contributor::NewContributorComponent;
use crate::component::new_funder::NewFunderComponent;
use crate::component::new_imprint::NewImprintComponent;
use crate::component::new_publisher::NewPublisherComponent;
use crate::component::new_series::NewSeriesComponent;
use crate::component::new_work::NewWorkComponent;
use crate::component::publication::PublicationComponent;
use crate::component::publications::PublicationsComponent;
use crate::component::publisher::PublisherComponent;
use crate::component::publishers::PublishersComponent;
use crate::component::series::SeriesComponent;
use crate::component::serieses::SeriesesComponent;
use crate::component::work::WorkComponent;
use crate::component::works::WorksComponent;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct AdminComponent {
    props: Props,
    router: RouteAgentDispatcher<()>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    RedirectToLogin,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub route: AdminRoute,
    pub current_user: Option<AccountDetails>,
}

impl Component for AdminComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        AdminComponent {
            props,
            router: RouteAgentDispatcher::new(),
            link,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render && self.props.current_user.is_none() {
            self.link.send_message(Msg::RedirectToLogin);
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RedirectToLogin => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from(AppRoute::Login)));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        if self.props.current_user.is_none() {
            self.link.send_message(Msg::RedirectToLogin);
        }
        true
    }

    fn view(&self) -> Html {
        if self.props.current_user.is_some() {
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
                                AdminRoute::Admin => html!{<DashboardComponent current_user = &self.props.current_user />},
                                AdminRoute::Dashboard => html!{<DashboardComponent current_user = &self.props.current_user />},
                                AdminRoute::Works => html!{<WorksComponent current_user = &self.props.current_user />},
                                AdminRoute::Work(id) => html!{<WorkComponent work_id = id, current_user = &self.props.current_user />},
                                AdminRoute::NewWork => html!{<NewWorkComponent/>},
                                AdminRoute::Publishers => html!{<PublishersComponent current_user = &self.props.current_user />},
                                AdminRoute::Publisher(id) => html!{<PublisherComponent publisher_id = id />},
                                AdminRoute::NewPublisher => html!{<NewPublisherComponent/>},
                                AdminRoute::Imprints => html!{<ImprintsComponent current_user = &self.props.current_user />},
                                AdminRoute::Imprint(id) => html!{<ImprintComponent imprint_id = id, current_user = &self.props.current_user />},
                                AdminRoute::NewImprint => html!{<NewImprintComponent current_user = &self.props.current_user />},
                                AdminRoute::Funders => html!{<FundersComponent current_user = &self.props.current_user />},
                                AdminRoute::Funder(id) => html!{<FunderComponent funder_id = id />},
                                AdminRoute::NewFunder => html!{<NewFunderComponent/>},
                                AdminRoute::Publications => html!{<PublicationsComponent current_user = &self.props.current_user />},
                                AdminRoute::Publication(id) => html!{<PublicationComponent publication_id = id />},
                                AdminRoute::NewPublication => {
                                    html!{
                                        <article class="message is-info">
                                            <div class="message-body">
                                                { "New publications can be added directly to the work." }
                                            </div>
                                        </article>
                                    }
                                }
                                AdminRoute::Contributors => html!{<ContributorsComponent current_user = &self.props.current_user />},
                                AdminRoute::Contributor(id) => html!{<ContributorComponent contributor_id = id />},
                                AdminRoute::NewContributor => html!{<NewContributorComponent/>},
                                AdminRoute::Serieses => html!{<SeriesesComponent current_user = &self.props.current_user />},
                                AdminRoute::NewSeries => html!{<NewSeriesComponent current_user = &self.props.current_user />},
                                AdminRoute::Series(id) => html!{<SeriesComponent series_id = id, current_user = &self.props.current_user />},
                            }
                        }
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }
}
