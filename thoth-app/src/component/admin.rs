use thoth_api::account::model::AccountDetails;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
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
use crate::service::account::AccountService;
use crate::string::PERMISSIONS_ERROR;

pub struct AdminComponent {
    props: Props,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    RedirectToLogin,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub route: AdminRoute,
    pub current_user: Option<AccountDetails>,
}

impl Component for AdminComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        if !AccountService::new().is_loggedin() {
            link.send_message(Msg::RedirectToLogin);
        }

        AdminComponent {
            props,
            notification_bus: NotificationBus::dispatcher(),
            router: RouteAgentDispatcher::new(),
            link,
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
        if self.props != props {
            let old_details = self.props.current_user.clone();
            self.props = props;
            if self.props.current_user.is_none() {
                self.link.send_message(Msg::RedirectToLogin);
            } else {
                let new_permissions = self
                    .props
                    .current_user
                    .as_ref()
                    .unwrap()
                    .resource_access
                    .clone();
                // Raise an error if user's permission set is empty,
                // but avoid raising repeated errors if permissions are unchanged
                if new_permissions.restricted_to() == Some(vec![])
                    && (old_details.is_none()
                        || (old_details.as_ref().unwrap().resource_access != new_permissions))
                {
                    self.notification_bus.send(Request::NotificationBusMsg((
                        PERMISSIONS_ERROR.into(),
                        NotificationStatus::Danger,
                    )));
                }
            }
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        if self.props.current_user.is_some()
            && self
                .props
                .current_user
                .as_ref()
                .unwrap()
                .resource_access
                .restricted_to()
                != Some(vec![])
        {
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
                                AdminRoute::Admin => html!{<DashboardComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Dashboard => html!{<DashboardComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Works => html!{<WorksComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Work(id) => html!{<WorkComponent work_id = id, current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::NewWork => html!{<NewWorkComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Publishers => html!{<PublishersComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Publisher(id) => html!{<PublisherComponent publisher_id = id, current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::NewPublisher => html!{<NewPublisherComponent/>},
                                AdminRoute::Imprints => html!{<ImprintsComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Imprint(id) => html!{<ImprintComponent imprint_id = id, current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::NewImprint => html!{<NewImprintComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Funders => html!{<FundersComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Funder(id) => html!{<FunderComponent funder_id = id />},
                                AdminRoute::NewFunder => html!{<NewFunderComponent/>},
                                AdminRoute::Publications => html!{<PublicationsComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Publication(id) => html!{<PublicationComponent publication_id = id, current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::NewPublication => {
                                    html!{
                                        <article class="message is-info">
                                            <div class="message-body">
                                                { "New publications can be added directly to the work." }
                                            </div>
                                        </article>
                                    }
                                }
                                AdminRoute::Contributors => html!{<ContributorsComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Contributor(id) => html!{<ContributorComponent contributor_id = id />},
                                AdminRoute::NewContributor => html!{<NewContributorComponent/>},
                                AdminRoute::Serieses => html!{<SeriesesComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::NewSeries => html!{<NewSeriesComponent current_user = self.props.current_user.as_ref().unwrap() />},
                                AdminRoute::Series(id) => html!{<SeriesComponent series_id = id, current_user = self.props.current_user.as_ref().unwrap() />},
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
