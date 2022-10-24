use thoth_api::model::contribution::ContributionType;
use thoth_api::model::contributor::Contributor;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::institution::CountryCode;
use thoth_api::model::institution::Institution;
use thoth_api::model::language::LanguageCode;
use thoth_api::model::language::LanguageRelation;
use thoth_api::model::location::LocationPlatform;
use thoth_api::model::price::CurrencyCode;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::publisher::Publisher;
use thoth_api::model::series::SeriesType;
use thoth_api::model::subject::SubjectType;
use thoth_api::model::work::WorkStatus;
use thoth_api::model::work::WorkType;
use thoth_api::model::work_relation::RelationType;
use uuid::Uuid;
use yew::function_component;
use yew::html;
use yew::virtual_dom::VNode;
use yew::Callback;
use yew::Event;
use yew::FocusEvent;
use yew::InputEvent;
use yew::MouseEvent;
use yew::Properties;

use crate::models::contribution::ContributionTypeValues;
use crate::models::institution::CountryCodeValues;
use crate::models::language::LanguageCodeValues;
use crate::models::language::LanguageRelationValues;
use crate::models::location::LocationPlatformValues;
use crate::models::price::CurrencyCodeValues;
use crate::models::publication::PublicationTypeValues;
use crate::models::series::SeriesTypeValues;
use crate::models::subject::SubjectTypeValues;
use crate::models::work::WorkStatusValues;
use crate::models::work::WorkTypeValues;
use crate::models::work_relation::RelationTypeValues;
use crate::string::NO;
use crate::string::RELOAD_BUTTON;
use crate::string::YES;

#[derive(PartialEq, Properties)]
pub struct FormInputProps {
    pub label: String,
    pub value: String,
    pub input_type: String,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or_default]
    pub step: Option<String>,
    #[prop_or_default]
    pub min: Option<String>,
    #[prop_or(false)]
    pub deactivated: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormTextareaProps {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(false)]
    pub deactivated: bool,
    #[prop_or_default]
    pub help_text: String,
}

// Variant of FormTextInput which supports tooltips,
// prepended static buttons, or both together.
// Also supports deactivating the input.
#[derive(PartialEq, Properties)]
pub struct FormTextInputExtendedProps {
    pub label: String,
    pub value: String,
    #[prop_or_default]
    pub tooltip: String,
    #[prop_or_default]
    pub statictext: String,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onfocus: Callback<FocusEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(false)]
    pub deactivated: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormTextInputProps {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(false)]
    pub deactivated: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormUrlInputProps {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormDateInputProps {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormFloatInputProps {
    pub label: String,
    pub value: Option<f64>,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or_default]
    pub step: Option<String>,
    #[prop_or("0".to_string())]
    pub min: String,
    #[prop_or(false)]
    pub deactivated: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormNumberInputProps {
    pub label: String,
    pub value: Option<i32>,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or("0".to_string())]
    pub min: String,
    #[prop_or(false)]
    pub deactivated: bool,
    #[prop_or_default]
    pub help_text: String,
}

#[derive(PartialEq, Properties)]
pub struct FormWorkTypeSelectProps {
    pub label: String,
    pub data: Vec<WorkTypeValues>,
    // Subset of `data` list which should be deactivated, if any
    #[prop_or_default]
    pub deactivate: Vec<WorkType>,
    pub value: WorkType,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormWorkStatusSelectProps {
    pub label: String,
    pub data: Vec<WorkStatusValues>,
    pub value: WorkStatus,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormContributionTypeSelectProps {
    pub label: String,
    pub data: Vec<ContributionTypeValues>,
    pub value: ContributionType,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormPublicationTypeSelectProps {
    pub label: String,
    pub data: Vec<PublicationTypeValues>,
    pub value: PublicationType,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormSubjectTypeSelectProps {
    pub label: String,
    pub data: Vec<SubjectTypeValues>,
    pub value: SubjectType,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormSeriesTypeSelectProps {
    pub label: String,
    pub data: Vec<SeriesTypeValues>,
    pub value: SeriesType,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormLanguageCodeSelectProps {
    pub label: String,
    pub data: Vec<LanguageCodeValues>,
    pub value: LanguageCode,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormLanguageRelationSelectProps {
    pub label: String,
    pub data: Vec<LanguageRelationValues>,
    pub value: LanguageRelation,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormCurrencyCodeSelectProps {
    pub label: String,
    pub data: Vec<CurrencyCodeValues>,
    pub value: CurrencyCode,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormLocationPlatformSelectProps {
    pub label: String,
    pub data: Vec<LocationPlatformValues>,
    pub value: LocationPlatform,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormCountryCodeSelectProps {
    pub label: String,
    pub data: Vec<CountryCodeValues>,
    pub value: Option<CountryCode>,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormRelationTypeSelectProps {
    pub label: String,
    pub data: Vec<RelationTypeValues>,
    pub value: RelationType,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormBooleanSelectProps {
    pub label: String,
    pub value: bool,
    pub onchange: Callback<Event>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormImprintSelectProps {
    pub label: String,
    pub data: Vec<ImprintWithPublisher>,
    pub value: Option<Uuid>,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormPublisherSelectProps {
    pub label: String,
    pub data: Vec<Publisher>,
    pub value: Option<Uuid>,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormInstitutionSelectProps {
    pub label: String,
    pub data: Vec<Institution>,
    pub value: Uuid,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct FormContributorSelectProps {
    pub label: String,
    pub data: Vec<Contributor>,
    pub value: Uuid,
    pub onchange: Callback<Event>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(PartialEq, Properties)]
pub struct ReloaderProps {
    pub onclick: Callback<MouseEvent>,
}

#[function_component(FormInput)]
pub fn form_input(props: &FormInputProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <input
                    class="input"
                    type={ props.input_type.clone() }
                    placeholder={ props.label.clone() }
                    value={ props.value.clone() }
                    oninput={ props.oninput.clone() }
                    onblur={ props.onblur.clone() }
                    required={ props.required }
                    step={ props.step.clone() }
                    min={ props.min.clone() }
                    disabled={ props.deactivated }
                />
            </div>
            {
                if !props.help_text.is_empty() {
                    html! {
                        <p class="help">{ props.help_text.clone() }</p>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[function_component(FormTextarea)]
pub fn form_textarea(props: &FormTextareaProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <textarea
                    class="textarea"
                    placeholder={ props.label.clone() }
                    value={ props.value.clone().unwrap_or_else(|| "".to_string()) }
                    oninput={ props.oninput.clone() }
                    required={ props.required }
                    disabled={ props.deactivated }
                />
            </div>
            {
                if !props.help_text.is_empty() {
                    html! {
                        <p class="help">{ props.help_text.clone() }</p>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[function_component(FormTextInputExtended)]
pub fn form_text_input_extended(props: &FormTextInputExtendedProps) -> VNode {
    // Only display tooltip if its value is set.
    let optional_tooltip = match props.tooltip.is_empty() {
        true => None,
        false => Some(props.tooltip.clone()),
    };
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div
                class="field has-addons is-expanded has-tooltip-arrow has-tooltip-bottom has-tooltip-active"
                data-tooltip={ optional_tooltip }
            >
                {
                    // Only display static button if a static text value was provided.
                    if props.statictext.is_empty() {
                        html! {}
                    } else {
                        html! {
                            <button class="button is-static">{ &props.statictext }</button>
                        }
                    }
                }
                <input
                    class="input"
                    type="text"
                    placeholder={ props.label.clone() }
                    value={ props.value.clone() }
                    oninput={ props.oninput.clone() }
                    onfocus={ props.onfocus.clone() }
                    onblur={ props.onblur.clone() }
                    required={ props.required }
                    disabled={ props.deactivated }
                />
            </div>
            {
                if !props.help_text.is_empty() {
                    html! {
                        <p class="help">{ props.help_text.clone() }</p>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[function_component(FormTextInput)]
pub fn form_text_input(props: &FormTextInputProps) -> VNode {
    html! {
        <FormInput
            label={ props.label.clone() }
            value={ props.value.clone().unwrap_or_else(|| "".to_string()) }
            input_type="text"
            oninput={ props.oninput.clone() }
            onblur={ props.onblur.clone() }
            required={ props.required }
            deactivated={ props.deactivated }
            help_text={ props.help_text.clone() }
        />
    }
}

#[function_component(FormUrlInput)]
pub fn form_url_input(props: &FormUrlInputProps) -> VNode {
    html! {
        <FormInput
            label={ props.label.clone() }
            value={ props.value.clone().unwrap_or_else(|| "".to_string()) }
            input_type="url"
            oninput={ props.oninput.clone() }
            onblur={ props.onblur.clone() }
            required={ props.required }
            help_text={ props.help_text.clone() }
        />
    }
}

#[function_component(FormDateInput)]
pub fn form_date_input(props: &FormDateInputProps) -> VNode {
    html! {
        <FormInput
            label={ props.label.clone() }
            value={ props.value.clone().unwrap_or_else(|| "".to_string()) }
            input_type="date"
            oninput={ props.oninput.clone() }
            onblur={ props.onblur.clone() }
            required={ props.required }
            help_text={ props.help_text.clone() }
        />
    }
}

#[function_component(FormNumberInput)]
pub fn form_number_input(props: &FormNumberInputProps) -> VNode {
    html! {
        <FormInput
            label={ props.label.clone() }
            value={ props.value.unwrap_or(0).to_string() }
            input_type="number"
            oninput={ props.oninput.clone() }
            onblur={ props.onblur.clone() }
            required={ props.required }
            min={ props.min.clone() }
            deactivated={ props.deactivated }
            help_text={ props.help_text.clone() }
        />
    }
}

#[function_component(FormFloatInput)]
pub fn form_float_input(props: &FormFloatInputProps) -> VNode {
    html! {
        <FormInput
            label={ props.label.clone() }
            value={ props.value.unwrap_or(0.00).to_string() }
            input_type="number"
            oninput={ props.oninput.clone() }
            onblur={ props.onblur.clone() }
            required={ props.required }
            step={ props.step.clone() }
            min={ props.min.clone() }
            deactivated={ props.deactivated }
            help_text={ props.help_text.clone() }
        />
    }
}

#[function_component(FormWorkTypeSelect)]
pub fn form_work_type_select(props: &FormWorkTypeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select is-fullwidth">
                <select required={ props.required } onchange={ &props.onchange }>
                    { for props.data.iter().map(|i| props.render_worktype(i, &props.deactivate)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormWorkStatusSelect)]
pub fn form_work_status_select(props: &FormWorkStatusSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select is-fullwidth">
                <select required={ props.required } onchange={ &props.onchange }>
                    { for props.data.iter().map(|i| props.render_workstatus(i)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormContributionTypeSelect)]
pub fn form_contribution_type_select(props: &FormContributionTypeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|i| props.render_contributiontype(i)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormPublicationTypeSelect)]
pub fn form_publication_type_select(props: &FormPublicationTypeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|p| props.render_publicationtype(p)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormSubjectTypeSelect)]
pub fn form_subject_type_select(props: &FormSubjectTypeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|p| props.render_subjecttype(p)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormSeriesTypeSelect)]
pub fn form_series_type_select(props: &FormSeriesTypeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|s| props.render_seriestype(s)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormLanguageCodeSelect)]
pub fn form_language_code_select(props: &FormLanguageCodeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|l| props.render_languagecode(l)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormLanguageRelationSelect)]
pub fn form_language_relation_select(props: &FormLanguageRelationSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|l| props.render_languagerelation(l)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormCurrencyCodeSelect)]
pub fn form_currency_code_select(props: &FormCurrencyCodeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|c| props.render_currencycode(c)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormLocationPlatformSelect)]
pub fn form_location_platform_select(props: &FormLocationPlatformSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|l| props.render_locationplatform(l)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormCountryCodeSelect)]
pub fn form_country_code_select(props: &FormCountryCodeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    <option value="">{"Select Country"}</option>
                    { for props.data.iter().map(|c| props.render_countrycode(c)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormRelationTypeSelect)]
pub fn form_relation_type_select(props: &FormRelationTypeSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                >
                    { for props.data.iter().map(|r| props.render_relationtype(r)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormBooleanSelect)]
pub fn form_boolean_select(props: &FormBooleanSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select">
                <select
                    required={ props.required }
                    onchange={ &props.onchange }
                    onblur={ props.onblur.clone() }
                >
                    <option value={ true.to_string() } selected={ props.value }>
                        { YES }
                    </option>
                    <option value={ false.to_string() } selected={ !props.value }>
                        { NO }
                    </option>
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormImprintSelect)]
pub fn form_imprint_select(props: &FormImprintSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select is-fullwidth">
                <select required={ props.required } onchange={ &props.onchange }>
                    <option value="">{"Select Imprint"}</option>
                    { for props.data.iter().map(|i| props.render_imprint(i)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormPublisherSelect)]
pub fn form_publisher_select(props: &FormPublisherSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select is-fullwidth">
                <select required={ props.required } onchange={ &props.onchange }>
                    <option value="">{"Select Publisher"}</option>
                    { for props.data.iter().map(|p| props.render_publisher(p)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormInstitutionSelect)]
pub fn form_institution_select(props: &FormInstitutionSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select is-fullwidth">
                <select required={ props.required } onchange={ &props.onchange }>
                    <option value="" selected={props.value.is_nil()}>{"Select Institution"}</option>
                    { for props.data.iter().map(|i| props.render_institution(i)) }
                </select>
                </div>
            </div>
        </div>
    }
}

#[function_component(FormContributorSelect)]
pub fn form_contributor_select(props: &FormContributorSelectProps) -> VNode {
    html! {
        <div class="field">
            <label class="label">{ &props.label }</label>
            <div class="control is-expanded">
                <div class="select is-fullwidth">
                <select required={ props.required } onchange={ &props.onchange }>
                    <option value="" selected={props.value.is_nil()}>{"Select Contributor"}</option>
                    { for props.data.iter().map(|c| props.render_contributor(c)) }
                </select>
                </div>
            </div>
        </div>
    }
}

impl FormWorkTypeSelectProps {
    fn render_worktype(&self, w: &WorkTypeValues, deactivate: &[WorkType]) -> VNode {
        let deactivated = deactivate.contains(&w.name);
        let selected = w.name == self.value;
        html! {
            <option value={w.name.to_string()} selected={selected} disabled={deactivated}>
                {&w.name}
            </option>
        }
    }
}

impl FormWorkStatusSelectProps {
    fn render_workstatus(&self, w: &WorkStatusValues) -> VNode {
        html! {
            <option value={w.name.to_string()} selected={w.name == self.value}>
                {&w.name}
            </option>
        }
    }
}

impl FormContributionTypeSelectProps {
    fn render_contributiontype(&self, c: &ContributionTypeValues) -> VNode {
        html! {
            <option value={c.name.to_string()} selected={c.name == self.value}>
                {&c.name}
            </option>
        }
    }
}

impl FormPublicationTypeSelectProps {
    fn render_publicationtype(&self, p: &PublicationTypeValues) -> VNode {
        html! {
            <option value={p.name.to_string()} selected={p.name == self.value}>
                {&p.name}
            </option>
        }
    }
}

impl FormSubjectTypeSelectProps {
    fn render_subjecttype(&self, s: &SubjectTypeValues) -> VNode {
        html! {
            <option value={s.name.to_string()} selected={s.name == self.value}>
                {&s.name}
            </option>
        }
    }
}

impl FormSeriesTypeSelectProps {
    fn render_seriestype(&self, s: &SeriesTypeValues) -> VNode {
        html! {
            <option value={s.name.to_string()} selected={s.name == self.value}>
                {&s.name}
            </option>
        }
    }
}

impl FormLanguageCodeSelectProps {
    fn render_languagecode(&self, l: &LanguageCodeValues) -> VNode {
        html! {
            <option value={l.name.to_string()} selected={l.name == self.value}>
                {&l.name}
            </option>
        }
    }
}

impl FormLanguageRelationSelectProps {
    fn render_languagerelation(&self, l: &LanguageRelationValues) -> VNode {
        html! {
            <option value={l.name.to_string()} selected={l.name == self.value}>
                {&l.name}
            </option>
        }
    }
}

impl FormCurrencyCodeSelectProps {
    fn render_currencycode(&self, c: &CurrencyCodeValues) -> VNode {
        html! {
            <option value={c.name.to_string()} selected={c.name == self.value}>
                {&c.name}
            </option>
        }
    }
}

impl FormLocationPlatformSelectProps {
    fn render_locationplatform(&self, l: &LocationPlatformValues) -> VNode {
        html! {
            <option value={l.name.to_string()} selected={l.name == self.value}>
                {&l.name}
            </option>
        }
    }
}

impl FormCountryCodeSelectProps {
    fn render_countrycode(&self, c: &CountryCodeValues) -> VNode {
        if Some(c.name.clone()) == self.value {
            html! {
                <option value={c.name.to_string()} selected={ true }>
                    {&c.name}
                </option>
            }
        } else {
            html! {
                <option value={c.name.to_string()}>{&c.name}</option>
            }
        }
    }
}

impl FormRelationTypeSelectProps {
    fn render_relationtype(&self, r: &RelationTypeValues) -> VNode {
        html! {
            <option value={r.name.to_string()} selected={r.name == self.value}>
                {&r.name}
            </option>
        }
    }
}

impl FormImprintSelectProps {
    fn render_imprint(&self, i: &ImprintWithPublisher) -> VNode {
        let value = &self.value.unwrap_or_default();
        html! {
            <option value={i.imprint_id.to_string()} selected={&i.imprint_id == value}>
                {&i.imprint_name}
            </option>
        }
    }
}

impl FormPublisherSelectProps {
    fn render_publisher(&self, p: &Publisher) -> VNode {
        let value = &self.value.unwrap_or_default();
        html! {
            <option value={p.publisher_id.to_string()} selected={&p.publisher_id == value}>
                {&p.publisher_name}
            </option>
        }
    }
}

impl FormInstitutionSelectProps {
    fn render_institution(&self, i: &Institution) -> VNode {
        html! {
            <option value={i.institution_id.to_string()} selected={i.institution_id == self.value}>
                {&i.to_string()}
            </option>
        }
    }
}

impl FormContributorSelectProps {
    fn render_contributor(&self, c: &Contributor) -> VNode {
        html! {
            <option value={c.contributor_id.to_string()} selected={c.contributor_id == self.value}>
                {&c.to_string()}
            </option>
        }
    }
}

#[function_component(Loader)]
pub fn loader() -> VNode {
    html! {
        <div class="hero is-medium">
            <div class="hero-body">
                <div class="container has-text-centered">
                    <progress class="progress is-warning" max="100"></progress>
                </div>
            </div>
        </div>
    }
}

#[function_component(Reloader)]
pub fn reloader(props: &ReloaderProps) -> VNode {
    html! {
        <div class="buttons has-addons is-centered">
            <button
                class="button is-success is-large"
                onclick={ &props.onclick }
            >
                <span class="icon">
                    <i class="fas fa-sync"></i>
                </span>
                <span>{ RELOAD_BUTTON }</span>
            </button>
        </div>
    }
}
