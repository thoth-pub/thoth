#![allow(clippy::unnecessary_operation)]

use thoth_api::model::reference::Reference;
use thoth_api::model::{Doi, Isbn, DOI_DOMAIN};
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
use crate::component::utils::FormDateInput;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextInputExtended;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::models::reference::create_reference_mutation::CreateReferenceRequest;
use crate::models::reference::create_reference_mutation::CreateReferenceRequestBody;
use crate::models::reference::create_reference_mutation::PushActionCreateReference;
use crate::models::reference::create_reference_mutation::PushCreateReference;
use crate::models::reference::create_reference_mutation::Variables;
use crate::models::reference::reference_fields_query::FetchActionReferenceFields;
use crate::models::reference::reference_fields_query::FetchReferenceFields;
use crate::models::reference::update_reference_mutation::PushActionUpdateReference;
use crate::models::reference::update_reference_mutation::PushUpdateReference;
use crate::models::reference::update_reference_mutation::UpdateReferenceRequest;
use crate::models::reference::update_reference_mutation::UpdateReferenceRequestBody;
use crate::models::reference::update_reference_mutation::Variables as UpdateVariables;
use crate::models::GraphqlFieldList;
use crate::string::CANCEL_BUTTON;
use crate::string::REFERENCES_INFO;

use super::ToElementValue;
use super::ToOption;

pub struct ReferenceModalComponent {
    reference: Reference,
    // Track the user-entered DOI string, which may not be validly formatted
    doi: String,
    doi_warning: String,
    // Track the user-entered ISBN string, which may not be validly formatted
    isbn: String,
    isbn_warning: String,
    in_edit_mode: bool,
    create_reference: PushCreateReference,
    update_reference: PushUpdateReference,
    notification_bus: NotificationDispatcher,
    // Store props value locally in order to test whether it has been updated on props change
    show_modal_form: bool,
    fetch_reference_fields: FetchReferenceFields,
    reference_fields: GraphqlFieldList,
}

pub enum Msg {
    CloseModalForm,
    ToggleModalFormDisplay,
    SetReferenceCreateState(PushActionCreateReference),
    CreateReference,
    SetReferenceUpdateState(PushActionUpdateReference),
    UpdateReference,
    SetReferenceFieldsFetchState(FetchActionReferenceFields),
    GetReferenceFields,
    ChangeOrdinal(String),
    ChangeDoi(String),
    ChangeUnstructuredCitation(String),
    ChangeIssn(String),
    ChangeIsbn(String),
    ChangeJournalTitle(String),
    ChangeArticleTitle(String),
    ChangeSeriesTitle(String),
    ChangeVolumeTitle(String),
    ChangeEdition(String),
    ChangeAuthor(String),
    ChangeVolume(String),
    ChangeIssue(String),
    ChangeFirstPage(String),
    ChangeComponentNumber(String),
    ChangeStandardDesignator(String),
    ChangeStandardsBodyName(String),
    ChangeStandardsBodyAcronym(String),
    ChangeUrl(String),
    ChangePublicationDate(String),
    ChangeRetrievalDate(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub reference_under_edit: Option<Reference>,
    pub work_id: Uuid,
    pub show_modal_form: bool,
    pub add_reference: Callback<Reference>,
    pub update_reference: Callback<Reference>,
    pub close_modal_form: Callback<()>,
}

impl Component for ReferenceModalComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let in_edit_mode = false;
        let reference: Reference = Default::default();
        let doi = Default::default();
        let doi_warning = Default::default();
        let isbn = Default::default();
        let isbn_warning = Default::default();
        let create_reference = Default::default();
        let update_reference = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let show_modal_form = ctx.props().show_modal_form;
        let fetch_reference_fields = Default::default();
        let reference_fields = Default::default();

        ctx.link().send_message(Msg::GetReferenceFields);

        ReferenceModalComponent {
            reference,
            doi,
            doi_warning,
            isbn,
            isbn_warning,
            in_edit_mode,
            create_reference,
            update_reference,
            notification_bus,
            show_modal_form,
            fetch_reference_fields,
            reference_fields,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CloseModalForm => {
                // Prompt parent form to close this form by updating the props
                // (this will eventually cause this form to re-render)
                ctx.props().close_modal_form.emit(());
                false
            }
            Msg::ToggleModalFormDisplay => {
                self.in_edit_mode = ctx.props().reference_under_edit.is_some();
                if ctx.props().show_modal_form {
                    if let Some(reference) = ctx.props().reference_under_edit.clone() {
                        // editing an existing reference
                        self.reference = reference;
                    }
                    // Ensure DOI variable value is kept in sync with reference object.
                    self.doi = self.reference.doi.clone().unwrap_or_default().to_string();
                    // Clear DOI warning as the variable value is now valid by definition
                    // (self.reference.doi can only store valid DOIs)
                    self.doi_warning = Default::default();
                    // Ditto for ISBN
                    self.isbn = self.reference.isbn.clone().unwrap_or_default().to_string();
                    self.isbn_warning = Default::default();
                }
                true
            }
            Msg::SetReferenceCreateState(fetch_state) => {
                self.create_reference.apply(fetch_state);
                match self.create_reference.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_reference {
                        Some(r) => {
                            // Send newly-created reference to parent form to process
                            // (parent form is responsible for closing modal)
                            ctx.props().add_reference.emit(r.clone());
                            self.reference = Default::default(); // reset form
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::CloseModalForm);
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::CloseModalForm);
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateReference => {
                // Update reference object with common field-specific logic before saving
                self.prepare_for_submission();
                let body = CreateReferenceRequestBody {
                    variables: Variables {
                        work_id: ctx.props().work_id,
                        reference_ordinal: self.reference.reference_ordinal,
                        doi: self.reference.doi.clone(),
                        unstructured_citation: self.reference.unstructured_citation.clone(),
                        issn: self.reference.issn.clone(),
                        isbn: self.reference.isbn.clone(),
                        journal_title: self.reference.journal_title.clone(),
                        article_title: self.reference.article_title.clone(),
                        series_title: self.reference.series_title.clone(),
                        volume_title: self.reference.volume_title.clone(),
                        edition: self.reference.edition,
                        author: self.reference.author.clone(),
                        volume: self.reference.volume.clone(),
                        issue: self.reference.issue.clone(),
                        first_page: self.reference.first_page.clone(),
                        component_number: self.reference.component_number.clone(),
                        standard_designator: self.reference.standard_designator.clone(),
                        standards_body_name: self.reference.standards_body_name.clone(),
                        standards_body_acronym: self.reference.standards_body_acronym.clone(),
                        url: self.reference.url.clone(),
                        publication_date: self.reference.publication_date,
                        retrieval_date: self.reference.retrieval_date,
                    },
                    ..Default::default()
                };
                let request = CreateReferenceRequest { body };
                self.create_reference = Fetch::new(request);
                ctx.link()
                    .send_future(self.create_reference.fetch(Msg::SetReferenceCreateState));
                ctx.link()
                    .send_message(Msg::SetReferenceCreateState(FetchAction::Fetching));
                false
            }
            Msg::SetReferenceUpdateState(fetch_state) => {
                self.update_reference.apply(fetch_state);
                match self.update_reference.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_reference {
                        Some(r) => {
                            // Send newly-created reference to parent form to process
                            // (parent form is responsible for closing modal)
                            ctx.props().update_reference.emit(r.clone());
                            self.reference = Default::default(); // reset form
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::CloseModalForm);
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::CloseModalForm);
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateReference => {
                // Update reference object with common field-specific logic before saving
                self.prepare_for_submission();
                let body = UpdateReferenceRequestBody {
                    variables: UpdateVariables {
                        reference_id: self.reference.reference_id,
                        work_id: ctx.props().work_id,
                        reference_ordinal: self.reference.reference_ordinal,
                        doi: self.reference.doi.clone(),
                        unstructured_citation: self.reference.unstructured_citation.clone(),
                        issn: self.reference.issn.clone(),
                        isbn: self.reference.isbn.clone(),
                        journal_title: self.reference.journal_title.clone(),
                        article_title: self.reference.article_title.clone(),
                        series_title: self.reference.series_title.clone(),
                        volume_title: self.reference.volume_title.clone(),
                        edition: self.reference.edition,
                        author: self.reference.author.clone(),
                        volume: self.reference.volume.clone(),
                        issue: self.reference.issue.clone(),
                        first_page: self.reference.first_page.clone(),
                        component_number: self.reference.component_number.clone(),
                        standard_designator: self.reference.standard_designator.clone(),
                        standards_body_name: self.reference.standards_body_name.clone(),
                        standards_body_acronym: self.reference.standards_body_acronym.clone(),
                        url: self.reference.url.clone(),
                        publication_date: self.reference.publication_date,
                        retrieval_date: self.reference.retrieval_date,
                    },
                    ..Default::default()
                };
                let request = UpdateReferenceRequest { body };
                self.update_reference = Fetch::new(request);
                ctx.link()
                    .send_future(self.update_reference.fetch(Msg::SetReferenceUpdateState));
                ctx.link()
                    .send_message(Msg::SetReferenceUpdateState(FetchAction::Fetching));
                false
            }
            Msg::SetReferenceFieldsFetchState(fetch_state) => {
                self.fetch_reference_fields.apply(fetch_state);
                self.reference_fields = match self.fetch_reference_fields.as_ref().state() {
                    FetchState::Fetched(body) => body.data.reference_fields.clone(),
                    _ => GraphqlFieldList::default(),
                };
                true
            }
            Msg::GetReferenceFields => {
                ctx.link().send_future(
                    self.fetch_reference_fields
                        .fetch(Msg::SetReferenceFieldsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetReferenceFieldsFetchState(FetchAction::Fetching));
                false
            }
            Msg::ChangeOrdinal(value) => {
                let ordinal = value.parse::<i32>().unwrap_or(0);
                self.reference.reference_ordinal.neq_assign(ordinal);
                false
            }
            Msg::ChangeDoi(value) => {
                if self.doi.neq_assign(value.trim().to_owned()) {
                    // If DOI is not correctly formatted, display a warning.
                    // Don't update self.reference.doi yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.doi_warning.clear();
                    match self.doi.parse::<Doi>() {
                        Err(e) => {
                            match e {
                                // If no DOI was provided, no warning is required.
                                ThothError::DoiEmptyError => {}
                                _ => self.doi_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.doi = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeUnstructuredCitation(value) => self
                .reference
                .unstructured_citation
                .neq_assign(value.to_opt_string()),
            Msg::ChangeIsbn(value) => {
                if self.isbn.neq_assign(value.trim().to_owned()) {
                    // If ISBN is not correctly formatted, display a warning.
                    // Don't update self.reference.isbn yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.isbn_warning.clear();
                    match self.isbn.parse::<Isbn>() {
                        Err(e) => {
                            match e {
                                // If no ISBN was provided, no warning is required.
                                ThothError::IsbnEmptyError => {}
                                _ => self.isbn_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.isbn = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeIssn(value) => self.reference.issn.neq_assign(value.to_opt_string()),
            Msg::ChangeJournalTitle(value) => self
                .reference
                .journal_title
                .neq_assign(value.to_opt_string()),
            Msg::ChangeArticleTitle(value) => self
                .reference
                .article_title
                .neq_assign(value.to_opt_string()),
            Msg::ChangeSeriesTitle(value) => self
                .reference
                .series_title
                .neq_assign(value.to_opt_string()),
            Msg::ChangeVolumeTitle(value) => self
                .reference
                .volume_title
                .neq_assign(value.to_opt_string()),
            Msg::ChangeEdition(value) => self.reference.edition.neq_assign(value.to_opt_int()),
            Msg::ChangeAuthor(value) => self.reference.author.neq_assign(value.to_opt_string()),
            Msg::ChangeVolume(value) => self.reference.volume.neq_assign(value.to_opt_string()),
            Msg::ChangeIssue(value) => self.reference.issue.neq_assign(value.to_opt_string()),
            Msg::ChangeFirstPage(value) => {
                self.reference.first_page.neq_assign(value.to_opt_string())
            }
            Msg::ChangeComponentNumber(value) => self
                .reference
                .component_number
                .neq_assign(value.to_opt_string()),
            Msg::ChangeStandardDesignator(value) => self
                .reference
                .standard_designator
                .neq_assign(value.to_opt_string()),
            Msg::ChangeStandardsBodyName(value) => self
                .reference
                .standards_body_name
                .neq_assign(value.to_opt_string()),
            Msg::ChangeStandardsBodyAcronym(value) => self
                .reference
                .standards_body_acronym
                .neq_assign(value.to_opt_string()),
            Msg::ChangeUrl(value) => self.reference.url.neq_assign(value.to_opt_string()),
            Msg::ChangePublicationDate(value) => self
                .reference
                .publication_date
                .neq_assign(value.to_opt_date()),
            Msg::ChangeRetrievalDate(value) => self
                .reference
                .retrieval_date
                .neq_assign(value.to_opt_date()),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_show_modal_form = self.show_modal_form.neq_assign(ctx.props().show_modal_form);
        if updated_show_modal_form {
            ctx.link().send_message(Msg::ToggleModalFormDisplay)
        }
        // Re-render only required if show_modal_form has changed,
        // in which case ToggleModalFormDisplay will trigger it
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::CloseModalForm
        });
        html! {
            <div class={ self.modal_form_status(ctx) }>
                <div class="modal-background" onclick={ &close_modal }></div>
                <div class="modal-card" style="width: 840px;">
                    <header class="modal-card-head">
                        <p class="modal-card-title">{ self.modal_form_title() }</p>
                        <button
                            class="delete"
                            aria-label="close"
                            onclick={ &close_modal }
                        ></button>
                    </header>
                    <section class="modal-card-body">
                        <form id="reference-modal" onsubmit={ self.modal_form_action(ctx) }>

                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormNumberInput
                                        label = "Reference Number"
                                        value={ self.reference.reference_ordinal }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOrdinal(e.to_value())) }
                                        required = true
                                        min={ "1".to_string() }
                                        help_text={ self.reference_fields.get_description("referenceOrdinal") }
                                    />
                                    <FormTextInputExtended
                                        label = "DOI"
                                        statictext={ DOI_DOMAIN }
                                        value={ self.doi.clone() }
                                        tooltip={ self.doi_warning.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeDoi(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("doi") }
                                    />
                                </div>
                            </div>
                            <FormTextarea
                                label = "Unstructured Citation"
                                value={ self.reference.unstructured_citation.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeUnstructuredCitation(e.to_value())) }
                                help_text={ self.reference_fields.get_description("unstructuredCitation") }
                            />

                            <hr/>

                            <article class="message is-info is-small">
                                <div class="message-body">
                                    { REFERENCES_INFO }
                                </div>
                            </article>

                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormTextInput
                                        label = "Article Title"
                                        value={ self.reference.article_title.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeArticleTitle(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("articleTitle") }
                                    />
                                    <FormTextInput
                                        label = "Journal Title"
                                        value={ self.reference.journal_title.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeJournalTitle(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("journalTitle") }
                                    />
                                    <FormTextInput
                                        label = "ISSN"
                                        value={ self.reference.issn.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeIssn(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("issn") }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormTextInput
                                        label = "Volume Title"
                                        value={ self.reference.volume_title.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeVolumeTitle(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("volumeTitle") }
                                    />
                                    <FormNumberInput
                                        label = "Edition"
                                        value={ self.reference.edition }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeEdition(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("edition") }
                                    />
                                    <FormTextInputExtended
                                        label = "ISBN"
                                        value={ self.isbn.clone() }
                                        tooltip={ self.isbn_warning.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeIsbn(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("isbn") }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormTextInput
                                        label = "Author"
                                        value={ self.reference.author.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeAuthor(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("author") }
                                    />
                                    <FormTextInput
                                        label = "First Page"
                                        value={ self.reference.first_page.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFirstPage(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("firstPage") }
                                    />
                                    <FormTextInput
                                        label = "Component Number"
                                        value={ self.reference.component_number.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeComponentNumber(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("componentNumber") }
                                    />
                                    <FormDateInput
                                        label = "Publication Date"
                                        value={ self.reference.publication_date.to_value() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePublicationDate(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("publicationDate") }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormTextInput
                                        label = "Series Title"
                                        value={ self.reference.series_title.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeSeriesTitle(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("seriesTitle") }
                                    />
                                    <FormTextInput
                                        label = "Volume"
                                        value={ self.reference.volume.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeVolume(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("volume") }
                                    />
                                    <FormTextInput
                                        label = "Issue"
                                        value={ self.reference.issue.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeIssue(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("issue") }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormUrlInput
                                        label = "URL"
                                        value={ self.reference.url.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeUrl(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("url") }
                                    />
                                    <FormDateInput
                                        label = "Retrieval Date"
                                        value={ self.reference.retrieval_date.to_value() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeRetrievalDate(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("retrievalDate") }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormTextInput
                                        label = "Standard Designator"
                                        value={ self.reference.standard_designator.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeStandardDesignator(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("standardDesignator") }
                                    />
                                    <FormTextInput
                                        label = "Standards Body Name"
                                        value={ self.reference.standards_body_name.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeStandardsBodyName(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("standardsBodyName") }
                                    />
                                    <FormTextInput
                                        label = "Stds Body Acronym"
                                        value={ self.reference.standards_body_acronym.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeStandardsBodyAcronym(e.to_value())) }
                                        help_text={ self.reference_fields.get_description("standardsBodyAcronym") }
                                    />
                                </div>
                            </div>
                        </form>
                    </section>
                    <footer class="modal-card-foot">
                        <button
                            class="button is-success"
                            type="submit"
                            form="reference-modal"
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
        }
    }
}

impl ReferenceModalComponent {
    fn modal_form_status(&self, ctx: &Context<Self>) -> String {
        match ctx.props().show_modal_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn modal_form_title(&self) -> String {
        match self.in_edit_mode {
            true => "Edit Reference".to_string(),
            false => "New Reference".to_string(),
        }
    }

    fn modal_form_button(&self) -> String {
        match self.in_edit_mode {
            true => "Save Reference".to_string(),
            false => "Add Reference".to_string(),
        }
    }

    fn modal_form_action(&self, ctx: &Context<Self>) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateReference
            }),
            false => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateReference
            }),
        }
    }

    fn prepare_for_submission(&mut self) {
        // Only update the ISBN value with the current user-entered string
        // if it is validly formatted - otherwise keep the default.
        // If no ISBN was provided, no format check is required.
        if self.isbn.is_empty() {
            self.reference.isbn.neq_assign(None);
        } else if let Ok(result) = self.isbn.parse::<Isbn>() {
            self.reference.isbn.neq_assign(Some(result));
        }
        // Same applies to DOI
        if self.doi.is_empty() {
            self.reference.doi.neq_assign(None);
        } else if let Ok(result) = self.doi.parse::<Doi>() {
            self.reference.doi.neq_assign(Some(result));
        }
    }
}
