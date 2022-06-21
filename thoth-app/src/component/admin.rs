use thoth_api::account::model::AccountDetails;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yew_router::prelude::*;
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

pub struct AdminComponent {
    notification_bus: NotificationDispatcher,
    previous_route: AdminRoute,
}

pub enum Msg {
    RedirectToLogin,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub current_user: Option<AccountDetails>,
}

impl Component for AdminComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        if !AccountService::new().is_loggedin() {
            ctx.link().send_message(Msg::RedirectToLogin);
        }
        let previous_route = ctx.link().route().unwrap();

        AdminComponent {
            notification_bus: NotificationBus::dispatcher(),
            previous_route,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if ctx.props().current_user.is_some()
            && ctx
                .props()
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

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RedirectToLogin => {
                ctx.link().history().unwrap().push(AppRoute::Login);
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.previous_route.neq_assign(ctx.link().route().unwrap());
        if ctx.props().current_user.is_none() {
            ctx.link().send_message(Msg::RedirectToLogin);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if ctx.props().current_user.is_some()
            && ctx
                .props()
                .current_user
                .as_ref()
                .unwrap()
                .resource_access
                .restricted_to()
                != Some(vec![])
        {
            let current_user = ctx.props().current_user.clone().unwrap();
            let route: AdminRoute = ctx.link().route().unwrap();
            let previous_route = self.previous_route.clone();
            let render = Switch::render(move |r| {
                switch_admin(r, current_user.clone(), previous_route.clone())
            });

            html! {
                <div class="columns">
                    <div class="column">
                        <div class="container">
                            <MenuComponent { route } />
                        </div>
                    </div>
                    <div class="column is-four-fifths">
                        <div class="container">
                            <BrowserRouter>
                                <Switch<AdminRoute> { render } />
                            </BrowserRouter>
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }
}

fn switch_admin(
    route: &AdminRoute,
    current_user: AccountDetails,
    previous_route: AdminRoute,
) -> Html {
    match route {
        AdminRoute::Admin => html! {<DashboardComponent { current_user } />},
        AdminRoute::Dashboard => html! {<DashboardComponent { current_user } />},
        AdminRoute::Works => html! {<WorksComponent { current_user } />},
        AdminRoute::Books => html! {<BooksComponent { current_user } />},
        AdminRoute::Chapters => html! {<ChaptersComponent { current_user } />},
        AdminRoute::Work { id } => html! {<WorkComponent work_id={ *id } { current_user } />},
        AdminRoute::NewWork => html! {<NewWorkComponent { current_user } { previous_route } />},
        AdminRoute::Publishers => html! {<PublishersComponent { current_user } />},
        AdminRoute::Publisher { id } => {
            html! {<PublisherComponent publisher_id={ *id } { current_user } />}
        }
        AdminRoute::NewPublisher => html! {<NewPublisherComponent/>},
        AdminRoute::Imprints => html! {<ImprintsComponent { current_user } />},
        AdminRoute::Imprint { id } => {
            html! {<ImprintComponent imprint_id={ *id } { current_user } />}
        }
        AdminRoute::NewImprint => html! {<NewImprintComponent { current_user } />},
        AdminRoute::Institutions => html! {<InstitutionsComponent { current_user } />},
        AdminRoute::Institution { id } => html! {<InstitutionComponent institution_id={ *id } />},
        AdminRoute::NewInstitution => html! {<NewInstitutionComponent/>},
        AdminRoute::Publications => html! {<PublicationsComponent { current_user } />},
        AdminRoute::Publication { id } => {
            html! {<PublicationComponent publication_id={ *id } { current_user } />}
        }
        AdminRoute::NewPublication => {
            html! {
                <article class="message is-info">
                    <div class="message-body">
                        { "New publications can be added directly to the work." }
                    </div>
                </article>
            }
        }
        AdminRoute::Contributors => html! {<ContributorsComponent { current_user } />},
        AdminRoute::Contributor { id } => html! {<ContributorComponent contributor_id={ *id } />},
        AdminRoute::NewContributor => html! {<NewContributorComponent/>},
        AdminRoute::Serieses => html! {<SeriesesComponent { current_user } />},
        AdminRoute::NewSeries => html! {<NewSeriesComponent { current_user } />},
        AdminRoute::Series { id } => html! {<SeriesComponent series_id={ *id } { current_user } />},
        _ => html! {},
    }
}
