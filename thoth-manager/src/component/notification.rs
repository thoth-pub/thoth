use yew::Bridge;
use yew::Component;
use yew::ComponentLink;
use yew::Html;
use yew::ShouldRender;
use yew::agent::Bridged;
use yew::html;

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
    link: ComponentLink<Self>,
    _producer: Box<dyn Bridge<NotificationBus>>,
}

impl Component for NotificationComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::Add);
        let _producer = NotificationBus::bridge(callback);
        NotificationComponent {
            notifications: Vec::new(),
            link,
            _producer,
        }
    }


    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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

    fn view(&self) -> Html {
        html! {
            <div class="column is-one-quarter fixed-right">
                { for self.notifications.iter().enumerate().map(|n| self.render_notification(n)) }
            </div>
        }
    }
}

impl NotificationComponent {
    fn render_notification(&self, (idx, notification): (usize, &Notification)) -> Html {
        html! {
            <div class={format!("notification {}", &notification.status)}>
                <button
                    onclick=self.link.callback(move |_| Msg::Remove(idx))
                    class="delete"
                ></button>
                { &notification.message }
            </div>
        }
    }
}
