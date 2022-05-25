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
use yew::html;
use yew::virtual_dom::VNode;
use yew::Callback;
use yew::ChangeData;
use yew::FocusEvent;
use yew::InputData;
use yew::MouseEvent;
use yew::Properties;
use yewtil::Pure;
use yewtil::PureComponent;

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

pub type FormInput = Pure<PureInput>;
pub type FormTextarea = Pure<PureTextarea>;
pub type FormTextInputExtended = Pure<PureTextInputExtended>;
pub type FormTextInput = Pure<PureTextInput>;
pub type FormUrlInput = Pure<PureUrlInput>;
pub type FormDateInput = Pure<PureDateInput>;
pub type FormFloatInput = Pure<PureFloatInput>;
pub type FormNumberInput = Pure<PureNumberInput>;
pub type FormWorkTypeSelect = Pure<PureWorkTypeSelect>;
pub type FormWorkStatusSelect = Pure<PureWorkStatusSelect>;
pub type FormContributionTypeSelect = Pure<PureContributionTypeSelect>;
pub type FormPublicationTypeSelect = Pure<PurePublicationTypeSelect>;
pub type FormSeriesTypeSelect = Pure<PureSeriesTypeSelect>;
pub type FormSubjectTypeSelect = Pure<PureSubjectTypeSelect>;
pub type FormLanguageCodeSelect = Pure<PureLanguageCodeSelect>;
pub type FormLanguageRelationSelect = Pure<PureLanguageRelationSelect>;
pub type FormCurrencyCodeSelect = Pure<PureCurrencyCodeSelect>;
pub type FormLocationPlatformSelect = Pure<PureLocationPlatformSelect>;
pub type FormCountryCodeSelect = Pure<PureCountryCodeSelect>;
pub type FormRelationTypeSelect = Pure<PureRelationTypeSelect>;
pub type FormBooleanSelect = Pure<PureBooleanSelect>;
pub type FormImprintSelect = Pure<PureImprintSelect>;
pub type FormPublisherSelect = Pure<PurePublisherSelect>;
pub type FormInstitutionSelect = Pure<PureInstitutionSelect>;
pub type FormContributorSelect = Pure<PureContributorSelect>;
pub type Loader = Pure<PureLoader>;
pub type Reloader = Pure<PureReloader>;

#[derive(Clone, PartialEq, Properties)]
pub struct PureInput {
    pub label: String,
    pub value: String,
    pub input_type: String,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
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
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureTextarea {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(false)]
    pub deactivated: bool,
}

// Variant of PureTextInput which supports tooltips,
// prepended static buttons, or both together.
// Also supports deactivating the input.
#[derive(Clone, PartialEq, Properties)]
pub struct PureTextInputExtended {
    pub label: String,
    pub value: String,
    #[prop_or_default]
    pub tooltip: String,
    #[prop_or_default]
    pub statictext: String,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onfocus: Callback<FocusEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(false)]
    pub deactivated: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureTextInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or(false)]
    pub deactivated: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureUrlInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureDateInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureFloatInput {
    pub label: String,
    pub value: Option<f64>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
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
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureNumberInput {
    pub label: String,
    pub value: Option<i32>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
    #[prop_or("0".to_string())]
    pub min: String,
    #[prop_or(false)]
    pub deactivated: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureWorkTypeSelect {
    pub label: String,
    pub data: Vec<WorkTypeValues>,
    // Subset of `data` list which should be deactivated, if any
    #[prop_or_default]
    pub deactivate: Vec<WorkType>,
    pub value: WorkType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureWorkStatusSelect {
    pub label: String,
    pub data: Vec<WorkStatusValues>,
    pub value: WorkStatus,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureContributionTypeSelect {
    pub label: String,
    pub data: Vec<ContributionTypeValues>,
    pub value: ContributionType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PurePublicationTypeSelect {
    pub label: String,
    pub data: Vec<PublicationTypeValues>,
    pub value: PublicationType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureSubjectTypeSelect {
    pub label: String,
    pub data: Vec<SubjectTypeValues>,
    pub value: SubjectType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureSeriesTypeSelect {
    pub label: String,
    pub data: Vec<SeriesTypeValues>,
    pub value: SeriesType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLanguageCodeSelect {
    pub label: String,
    pub data: Vec<LanguageCodeValues>,
    pub value: LanguageCode,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLanguageRelationSelect {
    pub label: String,
    pub data: Vec<LanguageRelationValues>,
    pub value: LanguageRelation,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureCurrencyCodeSelect {
    pub label: String,
    pub data: Vec<CurrencyCodeValues>,
    pub value: CurrencyCode,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLocationPlatformSelect {
    pub label: String,
    pub data: Vec<LocationPlatformValues>,
    pub value: LocationPlatform,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureCountryCodeSelect {
    pub label: String,
    pub data: Vec<CountryCodeValues>,
    pub value: Option<CountryCode>,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureRelationTypeSelect {
    pub label: String,
    pub data: Vec<RelationTypeValues>,
    pub value: RelationType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureBooleanSelect {
    pub label: String,
    pub value: bool,
    pub onchange: Callback<ChangeData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureImprintSelect {
    pub label: String,
    pub data: Vec<ImprintWithPublisher>,
    pub value: Option<Uuid>,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PurePublisherSelect {
    pub label: String,
    pub data: Vec<Publisher>,
    pub value: Option<Uuid>,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureInstitutionSelect {
    pub label: String,
    pub data: Vec<Institution>,
    pub value: Option<Uuid>,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureContributorSelect {
    pub label: String,
    pub data: Vec<Contributor>,
    pub value: Uuid,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLoader {}

#[derive(Clone, PartialEq, Properties)]
pub struct PureReloader {
    pub onclick: Callback<MouseEvent>,
}

impl PureComponent for PureInput {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <input
                        class="input"
                        type={ self.input_type.clone() }
                        placeholder={ self.label.clone() }
                        value={ self.value.clone() }
                        oninput=self.oninput.clone()
                        onblur=self.onblur.clone()
                        required={ self.required }
                        step={ self.step.clone() }
                        min={ self.min.clone() }
                        disabled={ self.deactivated }
                    />
                </div>
            </div>
        }
    }
}

impl PureComponent for PureTextarea {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <textarea
                        class="textarea"
                        placeholder=self.label.clone()
                        value={ self.value.clone().unwrap_or_else(|| "".to_string()) }
                        oninput=self.oninput.clone()
                        required={ self.required }
                        disabled={ self.deactivated }
                    />
                </div>
            </div>
        }
    }
}

impl PureComponent for PureTextInputExtended {
    fn render(&self) -> VNode {
        // Only display tooltip if its value is set.
        let optional_tooltip = match self.tooltip.is_empty() {
            true => None,
            false => Some(self.tooltip.clone()),
        };
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div
                    class="field has-addons is-expanded has-tooltip-arrow has-tooltip-bottom has-tooltip-active"
                    data-tooltip={ optional_tooltip }
                >
                    {
                        // Only display static button if a static text value was provided.
                        if self.statictext.is_empty() {
                            html! {}
                        } else {
                            html! {
                                <button class="button is-static">{ &self.statictext }</button>
                            }
                        }
                    }
                    <input
                        class="input"
                        type="text"
                        placeholder={ self.label.clone() }
                        value={ self.value.clone() }
                        oninput=self.oninput.clone()
                        onfocus=self.onfocus.clone()
                        onblur=self.onblur.clone()
                        required={ self.required }
                        disabled={ self.deactivated }
                    />
                </div>
            </div>
        }
    }
}

impl PureComponent for PureTextInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=self.label.clone()
                value=self.value.clone().unwrap_or_else(|| "".to_string())
                input_type="text"
                oninput=self.oninput.clone()
                onblur=self.onblur.clone()
                required=self.required
                deactivated=self.deactivated
            />
        }
    }
}

impl PureComponent for PureUrlInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=self.label.clone()
                value=self.value.clone().unwrap_or_else(|| "".to_string())
                input_type="url"
                oninput=self.oninput.clone()
                onblur=self.onblur.clone()
                required=self.required
            />
        }
    }
}

impl PureComponent for PureDateInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=self.label.clone()
                value=self.value.clone().unwrap_or_else(|| "".to_string())
                input_type="date"
                oninput=self.oninput.clone()
                onblur=self.onblur.clone()
                required=self.required
            />
        }
    }
}

impl PureComponent for PureNumberInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=self.label.clone()
                value=self.value.unwrap_or(0).to_string()
                input_type="number"
                oninput=self.oninput.clone()
                onblur=self.onblur.clone()
                required=self.required
                min=self.min.clone()
                deactivated=self.deactivated
            />
        }
    }
}

impl PureComponent for PureFloatInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=self.label.clone()
                value=self.value.unwrap_or(0.00).to_string()
                input_type="number"
                oninput=self.oninput.clone()
                onblur=self.onblur.clone()
                required=self.required
                step=self.step.clone()
                min=self.min.clone()
                deactivated=self.deactivated
            />
        }
    }
}

impl PureComponent for PureWorkTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        { for self.data.iter().map(|i| self.render_worktype(i, &self.deactivate)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureWorkStatusSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        { for self.data.iter().map(|i| self.render_workstatus(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureContributionTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|i| self.render_contributiontype(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PurePublicationTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|p| self.render_publicationtype(p)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureSubjectTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|p| self.render_subjecttype(p)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureSeriesTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|s| self.render_seriestype(s)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureLanguageCodeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|l| self.render_languagecode(l)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureLanguageRelationSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|l| self.render_languagerelation(l)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureCurrencyCodeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|c| self.render_currencycode(c)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureLocationPlatformSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|l| self.render_locationplatform(l)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureCountryCodeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        <option value="">{"Select Country"}</option>
                        { for self.data.iter().map(|c| self.render_countrycode(c)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureRelationTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|r| self.render_relationtype(r)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureBooleanSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                        onblur=self.onblur.clone()
                    >
                        <option value=true.to_string() selected=self.value>
                            { YES }
                        </option>
                        <option value=false.to_string() selected=!self.value>
                            { NO }
                        </option>
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureImprintSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        <option value="">{"Select Imprint"}</option>
                        { for self.data.iter().map(|i| self.render_imprint(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PurePublisherSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        <option value="">{"Select Publisher"}</option>
                        { for self.data.iter().map(|p| self.render_publisher(p)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureInstitutionSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        <option value="">{"Select Institution"}</option>
                        { for self.data.iter().map(|i| self.render_institution(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureContributorSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        <option value="" selected={self.value.is_nil()}>{"Select Contributor"}</option>
                        { for self.data.iter().map(|i| self.render_contributor(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureWorkTypeSelect {
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

impl PureWorkStatusSelect {
    fn render_workstatus(&self, w: &WorkStatusValues) -> VNode {
        html! {
            <option value={w.name.to_string()} selected={w.name == self.value}>
                {&w.name}
            </option>
        }
    }
}

impl PureContributionTypeSelect {
    fn render_contributiontype(&self, c: &ContributionTypeValues) -> VNode {
        html! {
            <option value={c.name.to_string()} selected={c.name == self.value}>
                {&c.name}
            </option>
        }
    }
}

impl PurePublicationTypeSelect {
    fn render_publicationtype(&self, p: &PublicationTypeValues) -> VNode {
        html! {
            <option value={p.name.to_string()} selected={p.name == self.value}>
                {&p.name}
            </option>
        }
    }
}

impl PureSubjectTypeSelect {
    fn render_subjecttype(&self, s: &SubjectTypeValues) -> VNode {
        html! {
            <option value={s.name.to_string()} selected={s.name == self.value}>
                {&s.name}
            </option>
        }
    }
}

impl PureSeriesTypeSelect {
    fn render_seriestype(&self, s: &SeriesTypeValues) -> VNode {
        html! {
            <option value={s.name.to_string()} selected={s.name == self.value}>
                {&s.name}
            </option>
        }
    }
}

impl PureLanguageCodeSelect {
    fn render_languagecode(&self, l: &LanguageCodeValues) -> VNode {
        html! {
            <option value={l.name.to_string()} selected={l.name == self.value}>
                {&l.name}
            </option>
        }
    }
}

impl PureLanguageRelationSelect {
    fn render_languagerelation(&self, l: &LanguageRelationValues) -> VNode {
        html! {
            <option value={l.name.to_string()} selected={l.name == self.value}>
                {&l.name}
            </option>
        }
    }
}

impl PureCurrencyCodeSelect {
    fn render_currencycode(&self, c: &CurrencyCodeValues) -> VNode {
        html! {
            <option value={c.name.to_string()} selected={c.name == self.value}>
                {&c.name}
            </option>
        }
    }
}

impl PureLocationPlatformSelect {
    fn render_locationplatform(&self, l: &LocationPlatformValues) -> VNode {
        html! {
            <option value={l.name.to_string()} selected={l.name == self.value}>
                {&l.name}
            </option>
        }
    }
}

impl PureCountryCodeSelect {
    fn render_countrycode(&self, c: &CountryCodeValues) -> VNode {
        if Some(c.name.clone()) == self.value {
            html! {
                <option value={c.name.to_string()} selected=true>
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

impl PureRelationTypeSelect {
    fn render_relationtype(&self, r: &RelationTypeValues) -> VNode {
        html! {
            <option value={r.name.to_string()} selected={r.name == self.value}>
                {&r.name}
            </option>
        }
    }
}

impl PureImprintSelect {
    fn render_imprint(&self, i: &ImprintWithPublisher) -> VNode {
        let value = &self.value.unwrap_or_default();
        html! {
            <option value={i.imprint_id.to_string()} selected={&i.imprint_id == value}>
                {&i.imprint_name}
            </option>
        }
    }
}

impl PurePublisherSelect {
    fn render_publisher(&self, p: &Publisher) -> VNode {
        let value = &self.value.unwrap_or_default();
        html! {
            <option value={p.publisher_id.to_string()} selected={&p.publisher_id == value}>
                {&p.publisher_name}
            </option>
        }
    }
}

impl PureInstitutionSelect {
    fn render_institution(&self, i: &Institution) -> VNode {
        let value = &self.value.unwrap_or_default();
        html! {
            <option value={i.institution_id.to_string()} selected={&i.institution_id == value}>
                {&i.institution_name}
            </option>
        }
    }
}

impl PureContributorSelect {
    fn render_contributor(&self, c: &Contributor) -> VNode {
        html! {
            <option value={c.contributor_id.to_string()} selected={c.contributor_id == self.value}>
                {&c.to_string()}
            </option>
        }
    }
}

impl PureComponent for PureLoader {
    fn render(&self) -> VNode {
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
}

impl PureComponent for PureReloader {
    fn render(&self) -> VNode {
        html! {
            <div class="buttons has-addons is-centered">
                <button
                    class="button is-success is-large"
                    onclick=&self.onclick
                >
                    <span class="icon">
                        <i class="fas fa-sync"></i>
                    </span>
                    <span>{ RELOAD_BUTTON }</span>
                </button>
            </div>
        }
    }
}
