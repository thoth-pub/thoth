use std::str::FromStr;
use thoth_api::model::institution::CountryCode;
use thoth_api::model::institution::Institution;
use thoth_api::model::{Doi, Ror, DOI_DOMAIN, ROR_DOMAIN};
use thoth_errors::ThothError;
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
use crate::component::utils::FormCountryCodeSelect;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextInputExtended;
use crate::models::institution::country_codes_query::FetchActionCountryCodes;
use crate::models::institution::country_codes_query::FetchCountryCodes;
use crate::models::institution::create_institution_mutation::CreateInstitutionRequest;
use crate::models::institution::create_institution_mutation::CreateInstitutionRequestBody;
use crate::models::institution::create_institution_mutation::PushActionCreateInstitution;
use crate::models::institution::create_institution_mutation::PushCreateInstitution;
use crate::models::institution::create_institution_mutation::Variables;
use crate::models::institution::CountryCodeValues;
use crate::models::EditRoute;
use crate::string::SAVE_BUTTON;

use super::ToElementValue;

pub struct NewInstitutionComponent {
    institution: Institution,
    fetch_country_codes: FetchCountryCodes,
    // Track the user-entered DOI string, which may not be validly formatted
    institution_doi: String,
    institution_doi_warning: String,
    // Track the user-entered ROR string, which may not be validly formatted
    ror: String,
    ror_warning: String,
    push_institution: PushCreateInstitution,
    data: InstitutionFormData,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct InstitutionFormData {
    country_codes: Vec<CountryCodeValues>,
}

pub enum Msg {
    SetCountryCodesFetchState(FetchActionCountryCodes),
    GetCountryCodes,
    SetInstitutionPushState(PushActionCreateInstitution),
    CreateInstitution,
    ChangeInstitutionName(String),
    ChangeInstitutionDoi(String),
    ChangeRor(String),
    ChangeCountryCode(String),
}

impl Component for NewInstitutionComponent {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let push_institution = Default::default();
        let data: InstitutionFormData = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let institution: Institution = Default::default();
        let fetch_country_codes = Default::default();
        let institution_doi = Default::default();
        let institution_doi_warning = Default::default();
        let ror = Default::default();
        let ror_warning = Default::default();

        ctx.link().send_message(Msg::GetCountryCodes);

        NewInstitutionComponent {
            institution,
            fetch_country_codes,
            institution_doi,
            institution_doi_warning,
            ror,
            ror_warning,
            push_institution,
            data,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetCountryCodesFetchState(fetch_state) => {
                self.fetch_country_codes.apply(fetch_state);
                self.data.country_codes = match self.fetch_country_codes.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.country_codes.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetCountryCodes => {
                ctx.link().send_future(
                    self.fetch_country_codes
                        .fetch(Msg::SetCountryCodesFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetCountryCodesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetInstitutionPushState(fetch_state) => {
                self.push_institution.apply(fetch_state);
                match self.push_institution.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_institution {
                        Some(i) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", i.institution_name),
                                NotificationStatus::Success,
                            )));
                            ctx.link().history().unwrap().push(i.edit_route());
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
            Msg::CreateInstitution => {
                // Only update the DOI value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
                // If no DOI was provided, no format check is required.
                if self.institution_doi.is_empty() {
                    self.institution.institution_doi.neq_assign(None);
                } else if let Ok(result) = self.institution_doi.parse::<Doi>() {
                    self.institution.institution_doi.neq_assign(Some(result));
                }
                // Only update the ROR value with the current user-entered string
                // if it is validly formatted - otherwise keep the database version.
                // If no ROR was provided, no format check is required.
                if self.ror.is_empty() {
                    self.institution.ror.neq_assign(None);
                } else if let Ok(result) = self.ror.parse::<Ror>() {
                    self.institution.ror.neq_assign(Some(result));
                }
                let body = CreateInstitutionRequestBody {
                    variables: Variables {
                        institution_name: self.institution.institution_name.clone(),
                        institution_doi: self.institution.institution_doi.clone(),
                        ror: self.institution.ror.clone(),
                        country_code: self.institution.country_code.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateInstitutionRequest { body };
                self.push_institution = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_institution.fetch(Msg::SetInstitutionPushState));
                ctx.link()
                    .send_message(Msg::SetInstitutionPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeInstitutionName(institution_name) => self
                .institution
                .institution_name
                .neq_assign(institution_name.trim().to_owned()),
            Msg::ChangeInstitutionDoi(value) => {
                if self.institution_doi.neq_assign(value.trim().to_owned()) {
                    // If DOI is not correctly formatted, display a warning.
                    // Don't update self.institution.institution_doi yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.institution_doi_warning.clear();
                    match self.institution_doi.parse::<Doi>() {
                        Err(e) => {
                            match e {
                                // If no DOI was provided, no warning is required.
                                ThothError::DoiEmptyError => {}
                                _ => self.institution_doi_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.institution_doi = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeRor(value) => {
                if self.ror.neq_assign(value.trim().to_owned()) {
                    // If ROR is not correctly formatted, display a warning.
                    // Don't update self.institution.ror yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.ror_warning.clear();
                    match self.ror.parse::<Ror>() {
                        Err(e) => {
                            match e {
                                // If no ROR was provided, no warning is required.
                                ThothError::RorEmptyError => {}
                                _ => self.ror_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.ror = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeCountryCode(value) => self
                .institution
                .country_code
                .neq_assign(CountryCode::from_str(&value).ok()),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateInstitution
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New institution" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit={ callback }>
                    <FormTextInput
                        label = "Institution Name"
                        value={ self.institution.institution_name.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeInstitutionName(e.to_value())) }
                        required = true
                    />
                    <FormTextInputExtended
                        label = "Institution DOI"
                        statictext={ DOI_DOMAIN }
                        value={ self.institution_doi.clone() }
                        tooltip={ self.institution_doi_warning.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeInstitutionDoi(e.to_value())) }
                    />
                    <FormTextInputExtended
                        label = "ROR ID"
                        statictext={ ROR_DOMAIN }
                        value={ self.ror.clone() }
                        tooltip={ self.ror_warning.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeRor(e.to_value())) }
                    />
                    <FormCountryCodeSelect
                        label = "Country"
                        value={ self.institution.country_code.clone() }
                        data={ self.data.country_codes.clone() }
                        onchange={ ctx.link().callback(|e: Event| Msg::ChangeCountryCode(e.to_value())) }
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
