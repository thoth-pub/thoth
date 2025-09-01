#![allow(clippy::unnecessary_operation)]

use std::str::FromStr;
use thoth_api::model::biography::Biography;
use thoth_api::model::contribution::Contribution;
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::contributor::Contributor;
use thoth_api::model::MarkupFormat;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::affiliations_form::AffiliationsFormComponent;
use crate::component::contributor_select::ContributorSelectComponent;
use crate::component::utils::FormBooleanSelect;
use crate::component::utils::FormContributionTypeSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
// use crate::models::biography::biographies_by_contribution_query::BiographiesByContributionRequest;
// use crate::models::biography::biographies_by_contribution_query::BiographiesByContributionRequestBody;
use crate::models::biography::biographies_by_contribution_query::FetchActionBiographiesByContribution;
use crate::models::biography::biographies_by_contribution_query::FetchBiographiesByContribution;
// use crate::models::biography::biographies_by_contribution_query::Variables as BiographiesByContributionVariables;
use crate::models::biography::create_biography_mutation::CreateBiographyRequest;
use crate::models::biography::create_biography_mutation::CreateBiographyRequestBody;
use crate::models::biography::create_biography_mutation::PushActionCreateBiography;
use crate::models::biography::create_biography_mutation::PushCreateBiography;
use crate::models::biography::create_biography_mutation::Variables as CreateBiographyVariables;
use crate::models::biography::delete_biography_mutation::DeleteBiographyRequest;
use crate::models::biography::delete_biography_mutation::DeleteBiographyRequestBody;
use crate::models::biography::delete_biography_mutation::PushActionDeleteBiography;
use crate::models::biography::delete_biography_mutation::PushDeleteBiography;
use crate::models::biography::delete_biography_mutation::Variables as DeleteBiographyVariables;
use crate::models::biography::update_biography_mutation::PushActionUpdateBiography;
use crate::models::biography::update_biography_mutation::PushUpdateBiography;
use crate::models::biography::update_biography_mutation::UpdateBiographyRequest;
use crate::models::biography::update_biography_mutation::UpdateBiographyRequestBody;
use crate::models::biography::update_biography_mutation::Variables as UpdateBiographyVariables;
use crate::models::contribution::contribution_types_query::FetchActionContributionTypes;
use crate::models::contribution::contribution_types_query::FetchContributionTypes;
use crate::models::contribution::create_contribution_mutation::CreateContributionRequest;
use crate::models::contribution::create_contribution_mutation::CreateContributionRequestBody;
use crate::models::contribution::create_contribution_mutation::PushActionCreateContribution;
use crate::models::contribution::create_contribution_mutation::PushCreateContribution;
use crate::models::contribution::create_contribution_mutation::Variables as CreateVariables;
use crate::models::contribution::delete_contribution_mutation::DeleteContributionRequest;
use crate::models::contribution::delete_contribution_mutation::DeleteContributionRequestBody;
use crate::models::contribution::delete_contribution_mutation::PushActionDeleteContribution;
use crate::models::contribution::delete_contribution_mutation::PushDeleteContribution;
use crate::models::contribution::delete_contribution_mutation::Variables as DeleteVariables;
use crate::models::contribution::update_contribution_mutation::PushActionUpdateContribution;
use crate::models::contribution::update_contribution_mutation::PushUpdateContribution;
use crate::models::contribution::update_contribution_mutation::UpdateContributionRequest;
use crate::models::contribution::update_contribution_mutation::UpdateContributionRequestBody;
use crate::models::contribution::update_contribution_mutation::Variables as UpdateVariables;
use crate::models::contribution::ContributionTypeValues;
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_CONTRIBUTIONS;
use crate::string::NO;
use crate::string::REMOVE_BUTTON;
use crate::string::YES;

use super::ToElementValue;
use super::ToOption;

pub struct ContributionsFormComponent {
    data: ContributionsFormData,
    contribution: Contribution,
    show_modal_form: bool,
    in_edit_mode: bool,
    fetch_contribution_types: FetchContributionTypes,
    create_contribution: PushCreateContribution,
    delete_contribution: PushDeleteContribution,
    update_contribution: PushUpdateContribution,
    create_biography: PushCreateBiography,
    delete_biography: PushDeleteBiography,
    update_biography: PushUpdateBiography,
    fetch_biographies: FetchBiographiesByContribution,
    biography: Biography,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct ContributionsFormData {
    contribution_types: Vec<ContributionTypeValues>,
}

pub enum Msg {
    ToggleModalFormDisplay(bool, Option<Contribution>),
    SetContributionTypesFetchState(FetchActionContributionTypes),
    GetContributionTypes,
    SetBiographyCreateState(PushActionCreateBiography),
    SetContributionCreateState(PushActionCreateContribution),
    CreateContribution(FetchActionBiographiesByContribution),
    SetBiographyUpdateState(PushActionUpdateBiography),
    SetContributionUpdateState(PushActionUpdateContribution),
    UpdateContribution(FetchActionBiographiesByContribution),
    SetBiographyDeleteState(PushActionDeleteBiography),
    SetContributionDeleteState(PushActionDeleteContribution),
    DeleteContribution(Uuid, FetchActionBiographiesByContribution),
    AddContribution(Contributor),
    ChangeContributor(Contributor),
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeBiography(String),
    ChangeContributiontype(ContributionType),
    ChangeMainContribution(bool),
    ChangeOrdinal(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub contributions: Option<Vec<Contribution>>,
    pub work_id: Uuid,
    pub update_contributions: Callback<Option<Vec<Contribution>>>,
}

impl Component for ContributionsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let data: ContributionsFormData = Default::default();
        let contribution: Contribution = Default::default();
        let show_modal_form = false;
        let in_edit_mode = false;
        let fetch_contribution_types = Default::default();
        let create_contribution = Default::default();
        let delete_contribution = Default::default();
        let update_contribution = Default::default();
        let create_biography = Default::default();
        let delete_biography = Default::default();
        let update_biography = Default::default();
        let fetch_biographies = Default::default();
        let biography = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ctx.link().send_message(Msg::GetContributionTypes);

        ContributionsFormComponent {
            data,
            contribution,
            show_modal_form,
            in_edit_mode,
            fetch_contribution_types,
            create_contribution,
            delete_contribution,
            update_contribution,
            create_biography,
            delete_biography,
            update_biography,
            fetch_biographies,
            biography,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, c) => {
                self.show_modal_form = show_form;
                self.in_edit_mode = c.is_some();
                if show_form {
                    if let Some(contribution) = c {
                        // Editing existing contribution: load its current values.
                        self.contribution = contribution;
                    }
                }
                true
            }
            Msg::SetContributionTypesFetchState(fetch_state) => {
                self.fetch_contribution_types.apply(fetch_state);
                self.data.contribution_types = match self.fetch_contribution_types.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.contribution_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::SetBiographyCreateState(fetch_state) => {
                self.create_biography.apply(fetch_state);
                match self.create_biography.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_biography {
                        Some(_) => true,
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
            Msg::SetBiographyUpdateState(fetch_state) => {
                self.update_biography.apply(fetch_state);
                match self.update_biography.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_biography {
                        Some(_) => true,
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
            Msg::SetBiographyDeleteState(fetch_state) => {
                self.delete_biography.apply(fetch_state);
                match self.delete_biography.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_biography {
                        Some(_) => true,
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
            Msg::GetContributionTypes => {
                ctx.link().send_future(
                    self.fetch_contribution_types
                        .fetch(Msg::SetContributionTypesFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionCreateState(fetch_state) => {
                self.create_contribution.apply(fetch_state);
                match self.create_contribution.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_contribution {
                        Some(i) => {
                            let contribution = i.clone();
                            let mut contributions: Vec<Contribution> =
                                ctx.props().contributions.clone().unwrap_or_default();
                            contributions.push(contribution);
                            ctx.props().update_contributions.emit(Some(contributions));
                            if !self.biography.content.is_empty() {
                                let create_biography_request_body = CreateBiographyRequestBody {
                                    variables: CreateBiographyVariables {
                                        contribution_id: i.contribution_id,
                                        work_id: ctx.props().work_id,
                                        content: self.biography.content.clone(),
                                        canonical: self.biography.canonical,
                                        locale_code: self.biography.locale_code,
                                        markup_format: MarkupFormat::default(),
                                    },
                                    ..Default::default()
                                };
                                let create_biography_request = CreateBiographyRequest {
                                    body: create_biography_request_body,
                                };

                                self.create_biography = Fetch::new(create_biography_request);

                                ctx.link().send_future(
                                    self.create_biography.fetch(Msg::SetBiographyCreateState),
                                );
                                ctx.link().send_message(Msg::SetBiographyCreateState(
                                    FetchAction::Fetching,
                                ));
                            }
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link()
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateContribution(_fetch_state) => {
                // Create contribution
                let create_contribution_request_body = CreateContributionRequestBody {
                    variables: CreateVariables {
                        work_id: ctx.props().work_id,
                        contributor_id: self.contribution.contributor_id,
                        contribution_type: self.contribution.contribution_type,
                        main_contribution: self.contribution.main_contribution,
                        first_name: self.contribution.first_name.clone(),
                        last_name: self.contribution.last_name.clone(),
                        full_name: self.contribution.full_name.clone(),
                        contribution_ordinal: self.contribution.contribution_ordinal,
                    },
                    ..Default::default()
                };
                let create_contribution_request = CreateContributionRequest {
                    body: create_contribution_request_body,
                };
                self.create_contribution = Fetch::new(create_contribution_request);
                ctx.link().send_future(
                    self.create_contribution
                        .fetch(Msg::SetContributionCreateState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionCreateState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionUpdateState(fetch_state) => {
                self.update_contribution.apply(fetch_state);
                match self.update_contribution.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_contribution {
                        Some(c) => {
                            let mut contributions: Vec<Contribution> =
                                ctx.props().contributions.clone().unwrap_or_default();
                            if let Some(contribution) = contributions
                                .iter_mut()
                                .find(|cn| cn.contribution_id == c.contribution_id)
                            {
                                *contribution = c.clone();
                                ctx.props().update_contributions.emit(Some(contributions));
                            } else {
                                // This should not be possible: the updated contribution returned from the
                                // database does not match any of the locally-stored contribution data.
                                // Refreshing the page will reload the local data from the database.
                                self.notification_bus.send(Request::NotificationBusMsg((
                                    "Changes were saved but display failed to update. Refresh your browser to view current data.".to_string(),
                                    NotificationStatus::Warning,
                                )));
                            }
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link()
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateContribution(fetch_state) => {
                self.fetch_biographies.apply(fetch_state);
                match self.fetch_biographies.as_ref().state() {
                    FetchState::NotFetching(_) => return false,
                    FetchState::Fetching(_) => return false,
                    FetchState::Fetched(body) => {
                        match &body.data.biographies {
                            Some(biographies) => {
                                // Update biography
                                let biography = biographies[0].clone();
                                let update_biography_request_body = UpdateBiographyRequestBody {
                                    variables: UpdateBiographyVariables {
                                        biography_id: biography.biography_id,
                                        contribution_id: self.contribution.contribution_id,
                                        work_id: ctx.props().work_id,
                                        content: biography.content.clone(),
                                        canonical: biography.canonical,
                                        locale_code: biography.locale_code,
                                    },
                                    ..Default::default()
                                };
                                let update_biography_request = UpdateBiographyRequest {
                                    body: update_biography_request_body,
                                };

                                self.update_biography = Fetch::new(update_biography_request);

                                ctx.link().send_future(
                                    self.update_biography.fetch(Msg::SetBiographyUpdateState),
                                );
                                ctx.link().send_message(Msg::SetBiographyUpdateState(
                                    FetchAction::Fetching,
                                ));
                            }
                            None => {
                                self.notification_bus.send(Request::NotificationBusMsg((
                                    "Failed to save".to_string(),
                                    NotificationStatus::Danger,
                                )));
                                return false;
                            }
                        }
                    }
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        return false;
                    }
                }

                // Update contribution
                let update_contribution_request_body = UpdateContributionRequestBody {
                    variables: UpdateVariables {
                        contribution_id: self.contribution.contribution_id,
                        work_id: ctx.props().work_id,
                        contributor_id: self.contribution.contributor_id,
                        contribution_type: self.contribution.contribution_type,
                        main_contribution: self.contribution.main_contribution,
                        first_name: self.contribution.first_name.clone(),
                        last_name: self.contribution.last_name.clone(),
                        full_name: self.contribution.full_name.clone(),
                        contribution_ordinal: self.contribution.contribution_ordinal,
                    },
                    ..Default::default()
                };
                let update_contribution_request = UpdateContributionRequest {
                    body: update_contribution_request_body,
                };
                self.update_contribution = Fetch::new(update_contribution_request);
                ctx.link().send_future(
                    self.update_contribution
                        .fetch(Msg::SetContributionUpdateState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionUpdateState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionDeleteState(fetch_state) => {
                self.delete_contribution.apply(fetch_state);
                match self.delete_contribution.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_contribution {
                        Some(contribution) => {
                            let to_keep: Vec<Contribution> = ctx
                                .props()
                                .contributions
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|c| c.contribution_id != contribution.contribution_id)
                                .collect();
                            ctx.props().update_contributions.emit(Some(to_keep));
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
            Msg::DeleteContribution(contribution_id, fetch_state) => {
                self.fetch_biographies.apply(fetch_state);
                match self.fetch_biographies.as_ref().state() {
                    FetchState::NotFetching(_) => return false,
                    FetchState::Fetching(_) => return false,
                    FetchState::Fetched(body) => {
                        match &body.data.biographies {
                            Some(biographies) => {
                                // Delete biography
                                let biography = biographies[0].clone();
                                let delete_biography_request_body = DeleteBiographyRequestBody {
                                    variables: DeleteBiographyVariables {
                                        biography_id: biography.biography_id,
                                    },
                                    ..Default::default()
                                };
                                let delete_biography_request = DeleteBiographyRequest {
                                    body: delete_biography_request_body,
                                };

                                self.delete_biography = Fetch::new(delete_biography_request);

                                ctx.link().send_future(
                                    self.delete_biography.fetch(Msg::SetBiographyDeleteState),
                                );
                                ctx.link().send_message(Msg::SetBiographyDeleteState(
                                    FetchAction::Fetching,
                                ));
                            }
                            None => {
                                self.notification_bus.send(Request::NotificationBusMsg((
                                    "Failed to save".to_string(),
                                    NotificationStatus::Danger,
                                )));
                                return false;
                            }
                        }
                    }
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        return false;
                    }
                }

                // Delete contribution
                let delete_contribution_request_body = DeleteContributionRequestBody {
                    variables: DeleteVariables { contribution_id },
                    ..Default::default()
                };
                let delete_contribution_request = DeleteContributionRequest {
                    body: delete_contribution_request_body,
                };
                self.delete_contribution = Fetch::new(delete_contribution_request);
                ctx.link().send_future(
                    self.delete_contribution
                        .fetch(Msg::SetContributionDeleteState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddContribution(contributor) => {
                self.contribution.contributor_id = contributor.contributor_id;
                self.contribution.first_name = contributor.first_name;
                self.contribution.last_name = contributor.last_name;
                self.contribution.full_name = contributor.full_name;
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(true, None));
                true
            }
            Msg::ChangeContributor(contributor) => {
                self.contribution.contributor_id = contributor.contributor_id;
                // Update user-editable name fields to default to canonical name, if changed
                self.contribution
                    .first_name
                    .neq_assign(contributor.first_name.clone());
                self.contribution
                    .last_name
                    .neq_assign(contributor.last_name.clone());
                self.contribution
                    .full_name
                    .neq_assign(contributor.full_name.clone());
                true
            }
            Msg::ChangeFirstName(val) => {
                self.contribution.first_name.neq_assign(val.to_opt_string())
            }
            Msg::ChangeLastName(val) => self
                .contribution
                .last_name
                .neq_assign(val.trim().to_owned()),
            Msg::ChangeFullName(val) => self
                .contribution
                .full_name
                .neq_assign(val.trim().to_owned()),
            Msg::ChangeBiography(val) => self.biography.content.neq_assign(val.trim().to_owned()),
            Msg::ChangeContributiontype(val) => self.contribution.contribution_type.neq_assign(val),
            Msg::ChangeMainContribution(val) => self.contribution.main_contribution.neq_assign(val),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.contribution.contribution_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let contributions = ctx.props().contributions.clone().unwrap_or_default();

        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false, None)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Contributions" }
                </p>
                <div class="panel-block">
                    <ContributorSelectComponent callback={ctx.link().callback(Msg::AddContribution)} />
                </div>
                <div class={ self.modal_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ self.modal_form_title() }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="contributions-form" onsubmit={ self.modal_form_action(ctx) }>
                                <div class="field">
                                    <label class="label">{ "Contributor" }</label>
                                    <div class="control is-expanded">
                                        { &self.contribution.full_name }
                                    </div>
                                </div>
                                <ContributorSelectComponent callback={ctx.link().callback(Msg::ChangeContributor)} />
                                <FormTextInput
                                    label="Contributor's Given Name"
                                    value={ self.contribution.first_name.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFirstName(e.to_value())) }
                                />
                                <FormTextInput
                                    label="Contributor's Family Name"
                                    value={ self.contribution.last_name.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLastName(e.to_value())) }
                                    required = true
                                />
                                <FormTextInput
                                    label="Contributor's Full Name"
                                    value={ self.contribution.full_name.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFullName(e.to_value())) }
                                    required = true
                                />
                                <FormContributionTypeSelect
                                    label = "Contribution Type"
                                    value={ self.contribution.contribution_type }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeContributiontype(ContributionType::from_str(&e.to_value()).unwrap())
                                    ) }
                                    data={ self.data.contribution_types.clone() }
                                    required = true
                                />
                                <FormTextInput
                                    label="Biography"
                                    value={ self.biography.content.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeBiography(e.to_value())) }
                                />
                                <FormBooleanSelect
                                    label = "Main"
                                    value={ self.contribution.main_contribution }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeMainContribution(e.to_value() == "true")
                                    ) }
                                    required = true
                                />
                                <FormNumberInput
                                    label = "Contribution Ordinal"
                                    value={ self.contribution.contribution_ordinal }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOrdinal(e.to_value())) }
                                    required = true
                                    min={ "1".to_string() }
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="contributions-form"
                            >
                                { self.modal_form_button() }
                            </button>
                            <button
                                class="button"
                                onclick={ &close_modal }
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !contributions.is_empty() {
                        html!{{for contributions.iter().map(|c| self.render_contribution(ctx, c))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_CONTRIBUTIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl ContributionsFormComponent {
    fn modal_form_status(&self) -> String {
        match self.show_modal_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn modal_form_title(&self) -> String {
        match self.in_edit_mode {
            true => "Edit Contribution".to_string(),
            false => "New Contribution".to_string(),
        }
    }

    fn modal_form_button(&self) -> String {
        match self.in_edit_mode {
            true => "Save Contribution".to_string(),
            false => "Add Contribution".to_string(),
        }
    }

    fn modal_form_action(&self, ctx: &Context<Self>) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateContribution(FetchAction::Fetching)
            }),
            false => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateContribution(FetchAction::Fetching)
            }),
        }
    }

    fn render_contribution(&self, ctx: &Context<Self>, c: &Contribution) -> Html {
        let contribution = c.clone();
        let contribution_id = c.contribution_id;
        html! {
            <div class="panel-block field is-horizontal is-flex-wrap-wrap">
                <span class="panel-icon">
                    <i class="fas fa-user" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Full Name" }</label>
                        <div class="control is-expanded">
                            {&c.full_name}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Contribution Type" }</label>
                        <div class="control is-expanded">
                            {&c.contribution_type}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Biography" }</label>
                        <div class="control is-expanded">
                            {&self.biography.content.clone()}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Main" }</label>
                        <div class="control is-expanded">
                            {
                                match c.main_contribution {
                                    true => { YES },
                                    false => { NO }
                                }
                            }
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Contribution Ordinal" }</label>
                        <div class="control is-expanded">
                            {&c.contribution_ordinal.clone()}
                        </div>
                    </div>

                    <div class="field is-grouped is-grouped-right">
                        <div class="control">
                            <a
                                class="button is-success"
                                onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true, Some(contribution.clone()))) }
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteContribution(contribution_id, FetchAction::Fetching)) }
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
                <AffiliationsFormComponent
                    contribution_id={ c.contribution_id }
                />
            </div>
        }
    }
}
