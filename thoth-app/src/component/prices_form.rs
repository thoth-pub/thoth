use std::str::FromStr;
use thoth_api::model::price::CurrencyCode;
use thoth_api::model::price::Price;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormCurrencyCodeSelect;
use crate::component::utils::FormFloatInput;
use crate::models::price::create_price_mutation::CreatePriceRequest;
use crate::models::price::create_price_mutation::CreatePriceRequestBody;
use crate::models::price::create_price_mutation::PushActionCreatePrice;
use crate::models::price::create_price_mutation::PushCreatePrice;
use crate::models::price::create_price_mutation::Variables;
use crate::models::price::currency_codes_query::FetchActionCurrencyCodes;
use crate::models::price::currency_codes_query::FetchCurrencyCodes;
use crate::models::price::delete_price_mutation::DeletePriceRequest;
use crate::models::price::delete_price_mutation::DeletePriceRequestBody;
use crate::models::price::delete_price_mutation::PushActionDeletePrice;
use crate::models::price::delete_price_mutation::PushDeletePrice;
use crate::models::price::delete_price_mutation::Variables as DeleteVariables;
use crate::models::price::CurrencyCodeValues;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_PRICES;
use crate::string::REMOVE_BUTTON;

pub struct PricesFormComponent {
    props: Props,
    data: PricesFormData,
    new_price: Price,
    show_add_form: bool,
    fetch_currency_codes: FetchCurrencyCodes,
    push_price: PushCreatePrice,
    delete_price: PushDeletePrice,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct PricesFormData {
    currency_codes: Vec<CurrencyCodeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetCurrencyCodesFetchState(FetchActionCurrencyCodes),
    GetCurrencyCodes,
    SetPricePushState(PushActionCreatePrice),
    CreatePrice,
    SetPriceDeleteState(PushActionDeletePrice),
    DeletePrice(Uuid),
    ChangeCurrencyCode(CurrencyCode),
    ChangeUnitPrice(String),
    DoNothing,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub prices: Option<Vec<Price>>,
    pub publication_id: Uuid,
    pub update_prices: Callback<Option<Vec<Price>>>,
}

impl Component for PricesFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: PricesFormData = Default::default();
        let show_add_form = false;
        let new_price: Price = Default::default();
        let fetch_currency_codes = Default::default();
        let push_price = Default::default();
        let delete_price = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        link.send_message(Msg::GetCurrencyCodes);

        PricesFormComponent {
            props,
            data,
            new_price,
            show_add_form,
            fetch_currency_codes,
            push_price,
            delete_price,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetCurrencyCodesFetchState(fetch_state) => {
                self.fetch_currency_codes.apply(fetch_state);
                self.data.currency_codes = match self.fetch_currency_codes.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.currency_codes.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetCurrencyCodes => {
                self.link.send_future(
                    self.fetch_currency_codes
                        .fetch(Msg::SetCurrencyCodesFetchState),
                );
                self.link
                    .send_message(Msg::SetCurrencyCodesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetPricePushState(fetch_state) => {
                self.push_price.apply(fetch_state);
                match self.push_price.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_price {
                        Some(l) => {
                            let price = l.clone();
                            let mut prices: Vec<Price> =
                                self.props.prices.clone().unwrap_or_default();
                            prices.push(price);
                            self.props.update_prices.emit(Some(prices));
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link.send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreatePrice => {
                let body = CreatePriceRequestBody {
                    variables: Variables {
                        publication_id: self.props.publication_id,
                        currency_code: self.new_price.currency_code.clone(),
                        unit_price: self.new_price.unit_price,
                    },
                    ..Default::default()
                };
                let request = CreatePriceRequest { body };
                self.push_price = Fetch::new(request);
                self.link
                    .send_future(self.push_price.fetch(Msg::SetPricePushState));
                self.link
                    .send_message(Msg::SetPricePushState(FetchAction::Fetching));
                false
            }
            Msg::SetPriceDeleteState(fetch_state) => {
                self.delete_price.apply(fetch_state);
                match self.delete_price.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_price {
                        Some(price) => {
                            let to_keep: Vec<Price> = self
                                .props
                                .prices
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|p| p.price_id != price.price_id)
                                .collect();
                            self.props.update_prices.emit(Some(to_keep));
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
            Msg::DeletePrice(price_id) => {
                let body = DeletePriceRequestBody {
                    variables: DeleteVariables { price_id },
                    ..Default::default()
                };
                let request = DeletePriceRequest { body };
                self.delete_price = Fetch::new(request);
                self.link
                    .send_future(self.delete_price.fetch(Msg::SetPriceDeleteState));
                self.link
                    .send_message(Msg::SetPriceDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeCurrencyCode(code) => self.new_price.currency_code.neq_assign(code),
            Msg::ChangeUnitPrice(val) => {
                let unit_price: f64 = val.parse().unwrap_or(0.00);
                self.new_price.unit_price.neq_assign(unit_price)
            }
            Msg::DoNothing => false, // callbacks need to return a message
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let prices = self.props.prices.clone().unwrap_or_default();
        let open_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(true)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Prices" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick=open_modal
                    >
                        { "Add Price" }
                    </button>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Price" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::DoNothing
                            })
                            >
                                <FormCurrencyCodeSelect
                                    label = "Price Code"
                                    value=self.new_price.currency_code.clone()
                                    data=self.data.currency_codes.clone()
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeCurrencyCode(
                                                CurrencyCode::from_str(&value).unwrap()
                                            )
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormFloatInput
                                    label = "Unit Price"
                                    value=self.new_price.unit_price
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeUnitPrice(e.value))
                                    required = true
                                    step="0.01"
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::CreatePrice
                                })
                            >
                                { "Add Price" }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !prices.is_empty() {
                        html!{{for prices.iter().map(|p| self.render_price(p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_PRICES }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl PricesFormComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_price(&self, p: &Price) -> Html {
        let price_id = p.price_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-file-invoice-dollar" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Currency" }</label>
                        <div class="control is-expanded">
                            {&p.currency_code}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Price" }</label>
                        <div class="control is-expanded">
                            {&p.unit_price}
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::DeletePrice(price_id))
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
