use thoth_api::model::reference::Reference;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::reference_modal::ReferenceModalComponent;
use crate::models::reference::delete_reference_mutation::DeleteReferenceRequest;
use crate::models::reference::delete_reference_mutation::DeleteReferenceRequestBody;
use crate::models::reference::delete_reference_mutation::PushActionDeleteReference;
use crate::models::reference::delete_reference_mutation::PushDeleteReference;
use crate::models::reference::delete_reference_mutation::Variables as DeleteVariables;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_REFERENCES;
use crate::string::REMOVE_BUTTON;

pub struct ReferencesFormComponent {
    show_modal_form: bool,
    reference_under_edit: Option<Reference>,
    delete_reference: PushDeleteReference,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    ToggleModalFormDisplay(bool, Option<Reference>),
    AddReference(Reference),
    UpdateReference(Reference),
    SetReferenceDeleteState(PushActionDeleteReference),
    DeleteReference(Uuid),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub references: Option<Vec<Reference>>,
    pub work_id: Uuid,
    pub update_references: Callback<Option<Vec<Reference>>>,
}

impl Component for ReferencesFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        let show_modal_form = false;
        let reference_under_edit = Default::default();
        let delete_reference = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ReferencesFormComponent {
            show_modal_form,
            reference_under_edit,
            delete_reference,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, reference) => {
                self.show_modal_form = show_form;
                self.reference_under_edit = reference;
                true
            }
            Msg::AddReference(reference) => {
                // Child form has created a new reference - add it to list
                let mut references: Vec<Reference> =
                    ctx.props().references.clone().unwrap_or_default();
                references.push(reference);
                ctx.props().update_references.emit(Some(references));
                // Close child form
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(false, None));
                true
            }
            Msg::UpdateReference(r) => {
                // Child form has updated an existing reference - replace it in list
                let mut references: Vec<Reference> =
                    ctx.props().references.clone().unwrap_or_default();
                if let Some(reference) = references
                    .iter_mut()
                    .find(|re| re.reference_id == r.reference_id)
                {
                    *reference = r.clone();
                    ctx.props().update_references.emit(Some(references));
                } else {
                    // This should not be possible: the updated reference returned from the
                    // database does not match any of the locally-stored reference data.
                    // Refreshing the page will reload the local data from the database.
                    self.notification_bus.send(Request::NotificationBusMsg((
                        "Changes were saved but display failed to update. Refresh your browser to view current data.".to_string(),
                        NotificationStatus::Warning,
                    )));
                }
                // Close child form
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(false, None));
                true
            }
            Msg::SetReferenceDeleteState(fetch_state) => {
                self.delete_reference.apply(fetch_state);
                match self.delete_reference.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_reference {
                        Some(reference) => {
                            let to_keep: Vec<Reference> = ctx
                                .props()
                                .references
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|r| r.reference_id != reference.reference_id)
                                .collect();
                            ctx.props().update_references.emit(Some(to_keep));
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
            Msg::DeleteReference(reference_id) => {
                let body = DeleteReferenceRequestBody {
                    variables: DeleteVariables { reference_id },
                    ..Default::default()
                };
                let request = DeleteReferenceRequest { body };
                self.delete_reference = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_reference.fetch(Msg::SetReferenceDeleteState));
                ctx.link()
                    .send_message(Msg::SetReferenceDeleteState(FetchAction::Fetching));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let references = ctx.props().references.clone().unwrap_or_default();
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(true, None)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "References" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick={ open_modal }
                    >
                        { "Add Reference" }
                    </button>
                </div>
                {
                    if !references.is_empty() {
                        html!{{for references.iter().map(|p| self.render_reference(ctx, p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_REFERENCES }
                            </div>
                        }
                    }
                }
                <ReferenceModalComponent
                    reference_under_edit={ self.reference_under_edit.clone() }
                    work_id={ ctx.props().work_id }
                    show_modal_form={ self.show_modal_form }
                    add_reference={ ctx.link().callback(Msg::AddReference) }
                    update_reference={ ctx.link().callback(Msg::UpdateReference) }
                    close_modal_form={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(false, None)) }
                />
            </nav>
        }
    }
}

impl ReferencesFormComponent {
    fn render_reference(&self, ctx: &Context<Self>, r: &Reference) -> Html {
        let reference = r.clone();
        let reference_id = r.reference_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-quote-right" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 2em;">
                        <label class="label">{ "Reference Ordinal" }</label>
                        <div class="control is-expanded">
                            {&r.reference_ordinal}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "DOI" }</label>
                        <div class="control is-expanded">
                            {&r.doi.clone().unwrap_or_default()}
                        </div>
                    </div>

                    <div class="field" style="width: 14em;">
                        <label class="label">{ "Citation" }</label>
                        <div class="control is-expanded truncate">
                            {&r.unstructured_citation.clone().unwrap_or_default()}
                        </div>
                    </div>

                    <div class="field is-grouped is-grouped-right">
                        <div class="control">
                            <a
                                class="button is-success"
                                onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true, Some(reference.clone()))) }
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteReference(reference_id)) }
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
