use thoth_api::account::model::AccountDetails;
use thoth_api::model::LengthUnit;
use yew::html;
use yew::prelude::*;
use yew::services::storage::Area;
use yew::services::storage::StorageService;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::books::BooksComponent;
use crate::component::chapters::ChaptersComponent;
use crate::component::contributor::ContributorComponent;
use crate::component::contributors::ContributorsComponent;
use crate::component::dashboard::DashboardComponent;
use crate::component::imprint::ImprintComponent;
use crate::component::imprints::ImprintsComponent;
use crate::component::institution::InstitutionComponent;
use crate::component::institutions::InstitutionsComponent;
use crate::component::menu::MenuComponent;
use crate::component::new_contributor::NewContributorComponent;
use crate::component::new_imprint::NewImprintComponent;
use crate::component::new_institution::NewInstitutionComponent;
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
use crate::string::STORAGE_ERROR;
use crate::UNITS_KEY;

pub struct AdminComponent {
    props: Props,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
    link: ComponentLink<Self>,
    units_selection: LengthUnit,
    previous_route: AdminRoute,
}

pub enum Msg {
    RedirectToLogin,
    UpdateLengthUnit(LengthUnit),
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
        let mut units_selection: LengthUnit = Default::default();
        let previous_route = props.route.clone();
        let mut storage_service = StorageService::new(Area::Local).expect(STORAGE_ERROR);
        if let Ok(units_string) = storage_service.restore(UNITS_KEY) {
            if let Ok(units) = units_string.parse::<LengthUnit>() {
                units_selection = units;
            } else {
                // Couldn't parse stored units - overwrite them with default
                storage_service.store(UNITS_KEY, Ok(units_selection.to_string()));
            }
        } else {
            // No stored units found - store the default
            storage_service.store(UNITS_KEY, Ok(units_selection.to_string()));
        }

        AdminComponent {
            props,
            notification_bus: NotificationBus::dispatcher(),
            router: RouteAgentDispatcher::new(),
            link,
            units_selection,
            previous_route,
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.props.current_user.is_some()
            && self
                .props
                .current_user
                .as_ref()
                .unwrap()
                .resource_access
                .restricted_to()
                == Some(vec![])
        {
            // Raise an error if user's permission set is empty
            self.notification_bus.send(Request::NotificationBusMsg((
                PERMISSIONS_ERROR.into(),
                NotificationStatus::Danger,
            )));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RedirectToLogin => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from(AppRoute::Login)));
                false
            }
            Msg::UpdateLengthUnit(length_unit) => {
                if self.units_selection.neq_assign(length_unit) {
                    StorageService::new(Area::Local)
                        .expect(STORAGE_ERROR)
                        .store(UNITS_KEY, Ok(self.units_selection.to_string()));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.previous_route.neq_assign(self.props.route.clone());
            self.props = props;
            if self.props.current_user.is_none() {
                self.link.send_message(Msg::RedirectToLogin);
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
                            <MenuComponent route = self.props.route.clone() />
                        </div>
                    </div>
                    <div class="column is-four-fifths">
                        <div class="container">
                        {
                            match &self.props.route {
                                AdminRoute::Admin => html!{<DashboardComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Dashboard => html!{<DashboardComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Works => html!{<WorksComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Books => html!{<BooksComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Chapters => html!{<ChaptersComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Work(id) => html!{<WorkComponent work_id = *id current_user = self.props.current_user.clone().unwrap() units_selection = self.units_selection.clone() update_units_selection = self.link.callback(Msg::UpdateLengthUnit) />},
                                AdminRoute::NewWork => html!{<NewWorkComponent current_user = self.props.current_user.clone().unwrap() units_selection = self.units_selection.clone() update_units_selection = self.link.callback(Msg::UpdateLengthUnit) previous_route = self.previous_route.clone() />},
                                AdminRoute::Publishers => html!{<PublishersComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Publisher(id) => html!{<PublisherComponent publisher_id = *id current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::NewPublisher => html!{<NewPublisherComponent/>},
                                AdminRoute::Imprints => html!{<ImprintsComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Imprint(id) => html!{<ImprintComponent imprint_id= *id current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::NewImprint => html!{<NewImprintComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Institutions => html!{<InstitutionsComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Institution(id) => html!{<InstitutionComponent institution_id = *id />},
                                AdminRoute::NewInstitution => html!{<NewInstitutionComponent/>},
                                AdminRoute::Publications => html!{<PublicationsComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Publication(id) => html!{<PublicationComponent publication_id= *id current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::NewPublication => {
                                    html!{
                                        <article class="message is-info">
                                            <div class="message-body">
                                                { "New publications can be added directly to the work." }
                                            </div>
                                        </article>
                                    }
                                }
                                AdminRoute::Contributors => html!{<ContributorsComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Contributor(id) => html!{<ContributorComponent contributor_id = *id />},
                                AdminRoute::NewContributor => html!{<NewContributorComponent/>},
                                AdminRoute::Serieses => html!{<SeriesesComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::NewSeries => html!{<NewSeriesComponent current_user = self.props.current_user.clone().unwrap() />},
                                AdminRoute::Series(id) => html!{<SeriesComponent series_id = *id current_user = self.props.current_user.clone().unwrap() />},
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
