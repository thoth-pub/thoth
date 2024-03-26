use std::str::FromStr;
use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::series::Series;
use thoth_api::model::series::SeriesType;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yew_router::history::History;
use yew_router::prelude::RouterScopeExt;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormSeriesTypeSelect;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::models::imprint::imprints_query::FetchActionImprints;
use crate::models::imprint::imprints_query::FetchImprints;
use crate::models::imprint::imprints_query::ImprintsRequest;
use crate::models::imprint::imprints_query::ImprintsRequestBody;
use crate::models::imprint::imprints_query::Variables as ImprintsVariables;
use crate::models::series::create_series_mutation::CreateSeriesRequest;
use crate::models::series::create_series_mutation::CreateSeriesRequestBody;
use crate::models::series::create_series_mutation::PushActionCreateSeries;
use crate::models::series::create_series_mutation::PushCreateSeries;
use crate::models::series::create_series_mutation::Variables;
use crate::models::series::series_types_query::FetchActionSeriesTypes;
use crate::models::series::series_types_query::FetchSeriesTypes;
use crate::models::series::SeriesTypeValues;
use crate::models::EditRoute;
use crate::string::SAVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct NewSeriesComponent {
    series: Series,
    push_series: PushCreateSeries,
    data: SeriesFormData,
    fetch_imprints: FetchImprints,
    fetch_series_types: FetchSeriesTypes,
    notification_bus: NotificationDispatcher,
    // Store props value locally in order to test whether it has been updated on props change
    resource_access: AccountAccess,
}

#[derive(Default)]
struct SeriesFormData {
    imprints: Vec<ImprintWithPublisher>,
    series_types: Vec<SeriesTypeValues>,
}

pub enum Msg {
    SetImprintsFetchState(FetchActionImprints),
    GetImprints,
    SetSeriesTypesFetchState(FetchActionSeriesTypes),
    GetSeriesTypes,
    SetSeriesPushState(PushActionCreateSeries),
    CreateSeries,
    ChangeSeriesType(SeriesType),
    ChangeImprint(Uuid),
    ChangeSeriesName(String),
    ChangeIssnPrint(String),
    ChangeIssnDigital(String),
    ChangeSeriesUrl(String),
    ChangeSeriesDescription(String),
    ChangeSeriesCfpUrl(String),
}
#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub current_user: AccountDetails,
}

impl Component for NewSeriesComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let push_series = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let series: Series = Default::default();
        let data: SeriesFormData = Default::default();
        let fetch_imprints: FetchImprints = Default::default();
        let fetch_series_types: FetchSeriesTypes = Default::default();
        let resource_access = ctx.props().current_user.resource_access.clone();

        ctx.link().send_message(Msg::GetImprints);
        ctx.link().send_message(Msg::GetSeriesTypes);

        NewSeriesComponent {
            series,
            push_series,
            data,
            fetch_imprints,
            fetch_series_types,
            notification_bus,
            resource_access,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetImprintsFetchState(fetch_state) => {
                self.fetch_imprints.apply(fetch_state);
                self.data.imprints = match self.fetch_imprints.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.imprints.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetImprints => {
                let body = ImprintsRequestBody {
                    variables: ImprintsVariables {
                        publishers: ctx.props().current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = ImprintsRequest { body };
                self.fetch_imprints = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_imprints.fetch(Msg::SetImprintsFetchState));
                ctx.link()
                    .send_message(Msg::SetImprintsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetSeriesTypesFetchState(fetch_state) => {
                self.fetch_series_types.apply(fetch_state);
                self.data.series_types = match self.fetch_series_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.series_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetSeriesTypes => {
                ctx.link()
                    .send_future(self.fetch_series_types.fetch(Msg::SetSeriesTypesFetchState));
                ctx.link()
                    .send_message(Msg::SetSeriesTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetSeriesPushState(fetch_state) => {
                self.push_series.apply(fetch_state);
                match self.push_series.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_series {
                        Some(s) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", s.series_name),
                                NotificationStatus::Success,
                            )));
                            ctx.link().history().unwrap().push(s.edit_route());
                            true
                        }
                        None => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateSeries => {
                let body = CreateSeriesRequestBody {
                    variables: Variables {
                        series_type: self.series.series_type.clone(),
                        series_name: self.series.series_name.clone(),
                        issn_print: self.series.issn_print.clone(),
                        issn_digital: self.series.issn_digital.clone(),
                        series_url: self.series.series_url.clone(),
                        series_description: self.series.series_description.clone(),
                        series_cfp_url: self.series.series_cfp_url.clone(),
                        imprint_id: self.series.imprint_id,
                    },
                    ..Default::default()
                };
                let request = CreateSeriesRequest { body };
                self.push_series = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_series.fetch(Msg::SetSeriesPushState));
                ctx.link()
                    .send_message(Msg::SetSeriesPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeSeriesType(series_type) => self.series.series_type.neq_assign(series_type),
            Msg::ChangeImprint(imprint_id) => self.series.imprint_id.neq_assign(imprint_id),
            Msg::ChangeSeriesName(series_name) => self
                .series
                .series_name
                .neq_assign(series_name.trim().to_owned()),
            Msg::ChangeIssnPrint(issn_print) => self
                .series
                .issn_print
                .neq_assign(issn_print.to_opt_string()),
            Msg::ChangeIssnDigital(issn_digital) => self
                .series
                .issn_digital
                .neq_assign(issn_digital.to_opt_string()),
            Msg::ChangeSeriesUrl(value) => self.series.series_url.neq_assign(value.to_opt_string()),
            Msg::ChangeSeriesDescription(value) => self
                .series
                .series_description
                .neq_assign(value.to_opt_string()),
            Msg::ChangeSeriesCfpUrl(value) => {
                self.series.series_cfp_url.neq_assign(value.to_opt_string())
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access.clone());
        if updated_permissions {
            ctx.link().send_message(Msg::GetImprints);
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateSeries
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New series" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit={ callback }>
                    <FormSeriesTypeSelect
                        label = "Series Type"
                        value={ self.series.series_type.clone() }
                        onchange={ ctx.link().callback(|e: Event|
                            Msg::ChangeSeriesType(SeriesType::from_str(&e.to_value()).unwrap())
                        ) }
                        data={ self.data.series_types.clone() }
                        required = true
                    />
                    <FormImprintSelect
                        label = "Imprint"
                        value={ self.series.imprint_id }
                        data={ self.data.imprints.clone() }
                        onchange={ ctx.link().callback(|e: Event|
                            Msg::ChangeImprint(Uuid::parse_str(&e.to_value()).unwrap_or_default())
                        ) }
                        required = true
                    />
                    <FormTextInput
                        label = "Series Name"
                        value={ self.series.series_name.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeSeriesName(e.to_value())) }
                        required = true
                    />
                    <FormTextInput
                        label = "ISSN Print"
                        value={ self.series.issn_print.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeIssnPrint(e.to_value())) }
                    />
                    <FormTextInput
                        label = "ISSN Digital"
                        value={ self.series.issn_digital.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeIssnDigital(e.to_value())) }
                    />
                    <FormUrlInput
                        label = "Series URL"
                        value={ self.series.series_url.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeSeriesUrl(e.to_value())) }
                    />
                    <FormUrlInput
                        label = "Series Call for Proposals URL"
                        value={ self.series.series_cfp_url.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeSeriesCfpUrl(e.to_value())) }
                    />
                    <FormTextarea
                        label = "Series Description"
                        value={ self.series.series_description.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeSeriesDescription(e.to_value())) }
                    />

                    <div class="field">
                        <div class="control">
                            <button class="button is-success" type="submit">
                                { SAVE_BUTTON }
                            </button>
                        </div>
                    </div>
                </form>
            </>
        }
    }
}
