use yew::agent::Bridged;
use yew::html;
use yew::prelude::Context;
use yew::Bridge;
use yew::Component;
use yew::Html;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationStatus;

pub enum Msg {
    Add((String, NotificationStatus)),
    Remove(usize),
}

struct Notification {
    message: String,
    status: NotificationStatus,
}

pub struct NotificationComponent {
    notifications: Vec<Notification>,
    _producer: Box<dyn Bridge<NotificationBus>>,
}

impl Component for NotificationComponent {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::Add);
        let _producer = NotificationBus::bridge(callback);
        NotificationComponent {
            notifications: Vec::new(),
            _producer,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Add(s) => {
                let notification = Notification {
                    message: s.0,
                    status: s.1,
                };
                self.notifications.push(notification);
            }
            Msg::Remove(idx) => {
                self.notifications.remove(idx);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="column is-one-quarter fixed-right">
                { for self.notifications.iter().enumerate().map(|n| self.render_notification(ctx, n)) }
            </div>
        }
    }
}

impl NotificationComponent {
    fn render_notification(
        &self,
        ctx: &Context<Self>,
        (idx, notification): (usize, &Notification),
    ) -> Html {
        html! {
            <div class={format!("notification {}", &notification.status)}>
                <button
                    onclick={ ctx.link().callback(move |_| Msg::Remove(idx)) }
                    class="delete"
                ></button>
                { &notification.message }
            </div>
        }
    }
}
