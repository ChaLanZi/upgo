use dioxus::prelude::*;

#[component]
pub fn SettingsPage(on_logout: EventHandler<()>, user_id: String) -> Element {
    rsx! {
        div { class: "settings-container",
            h2 { "Account Settings" }
            p { "User ID: {user_id}" }
            hr {}
            button { class: "btn btn-danger", onclick: move |_| on_logout.call(()), "Logout All Devices" }
        }
    }
}
