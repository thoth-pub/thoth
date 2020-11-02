use std::str::FromStr;
use thoth_api::series::model::SeriesType;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormSeriesTypeSelect;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::models::imprint::imprints_query::FetchActionImprints;
use crate::models::imprint::imprints_query::FetchImprints;
use crate::models::imprint::Imprint;
use crate::models::series::create_series_mutation::CreateSeriesRequest;
use crate::models::series::create_series_mutation::CreateSeriesRequestBody;
use crate::models::series::create_series_mutation::PushActionCreateSeries;
use crate::models::series::create_series_mutation::PushCreateSeries;
use crate::models::series::create_series_mutation::Variables;
use crate::models::series::series_types_query::FetchActionSeriesTypes;
use crate::models::series::series_types_query::FetchSeriesTypes;
use crate::models::series::Series;
use crate::models::series::SeriesTypeValues;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewSeriesComponent {
    series: Series,
    push_series: PushCreateSeries,
    data: SeriesFormData,
    fetch_imprints: FetchImprints,
    fetch_series_types: FetchSeriesTypes,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct SeriesFormData {
    imprints: Vec<Imprint>,
    series_types: Vec<SeriesTypeValues>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    SetImprintsFetchState(FetchActionImprints),
    GetImprints,
    SetSeriesTypesFetchState(FetchActionSeriesTypes),
    GetSeriesTypes,
    SetSeriesPushState(PushActionCreateSeries),
    CreateSeries,
    ChangeSeriesType(SeriesType),
    ChangeImprint(String),
    ChangeSeriesName(String),
    ChangeIssnPrint(String),
    ChangeIssnDigital(String),
    ChangeSeriesUrl(String),
    ChangeRoute(AppRoute),
}

impl Component for NewSeriesComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_series = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let series: Series = Default::default();
        let data: SeriesFormData = Default::default();
        let fetch_imprints: FetchImprints = Default::default();
        let fetch_series_types: FetchSeriesTypes = Default::default();
        let router = RouteAgentDispatcher::new();

        link.send_message(Msg::GetImprints);
        link.send_message(Msg::GetSeriesTypes);

        NewSeriesComponent {
            series,
            push_series,
            data,
            fetch_imprints,
            fetch_series_types,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
                self.link
                    .send_future(self.fetch_imprints.fetch(Msg::SetImprintsFetchState));
                self.link
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
                self.link
                    .send_future(self.fetch_series_types.fetch(Msg::SetSeriesTypesFetchState));
                self.link
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
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(AdminRoute::Series(s.series_id.clone()))));
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
                            err.to_string(),
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
                        imprint_id: self.series.imprint.imprint_id.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateSeriesRequest { body };
                self.push_series = Fetch::new(request);
                self.link
                    .send_future(self.push_series.fetch(Msg::SetSeriesPushState));
                self.link
                    .send_message(Msg::SetSeriesPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeSeriesType(series_type) => self.series.series_type.neq_assign(series_type),
            Msg::ChangeImprint(imprint_id) => self.series.imprint.imprint_id.neq_assign(imprint_id),
            Msg::ChangeSeriesName(series_name) => self.series.series_name.neq_assign(series_name),
            Msg::ChangeIssnPrint(issn_print) => self.series.issn_print.neq_assign(issn_print),
            Msg::ChangeIssnDigital(issn_digital) => {
                self.series.issn_digital.neq_assign(issn_digital)
            }
            Msg::ChangeSeriesUrl(series_url) => self.series.series_url.neq_assign(Some(series_url)),
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let callback = self.link.callback(|event: FocusEvent| {
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

                <form onsubmit=callback>
                    <FormSeriesTypeSelect
                        label = "Series Type"
                        value=&self.series.series_type
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                Msg::ChangeSeriesType(SeriesType::from_str(&value).unwrap())
                            }
                            _ => unreachable!(),
                        })
                        data=&self.data.series_types
                        required = true
                    />
                    <FormImprintSelect
                        label = "Imprint"
                        value=&self.series.imprint.imprint_id
                        data=&self.data.imprints
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                Msg::ChangeImprint(value.clone())
                            }
                            _ => unreachable!(),
                        })
                        required = true
                    />
                    <FormTextInput
                        label = "Series Name"
                        value=&self.series.series_name
                        oninput=self.link.callback(|e: InputData| Msg::ChangeSeriesName(e.value))
                        required=true
                    />
                    <FormTextInput
                        label = "ISSN Print"
                        value=&self.series.issn_print
                        oninput=self.link.callback(|e: InputData| Msg::ChangeIssnPrint(e.value))
                        required=true
                    />
                    <FormTextInput
                        label = "ISSN Digital"
                        value=&self.series.issn_digital
                        oninput=self.link.callback(|e: InputData| Msg::ChangeIssnDigital(e.value))
                        required=true
                    />
                    <FormUrlInput
                        label = "Series URL"
                        value=&self.series.series_url
                        oninput=self.link.callback(|e: InputData| Msg::ChangeSeriesUrl(e.value))
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
