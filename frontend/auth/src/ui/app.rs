//! Minimal Dioxus app — just renders the login UI as MVP.

use dioxus::prelude::*;

const STYLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/style.css"));

#[component]
pub fn App() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut msg = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: STYLE }
        div { class: "app-container",
            div { class: "auth-container",
                div { class: "auth-card",
                    h1 { class: "auth-title", "Upgo" }
                    p { class: "auth-subtitle", "Sign in" }

                    if !msg().is_empty() {
                        div { class: "alert alert-info", "{msg}" }
                    }

                    div { class: "form-group",
                        label { "Email" }
                        input {
                            class: "form-input", r#type: "email",
                            placeholder: "your@email.com",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }
                    div { class: "form-group",
                        label { "Password" }
                        input {
                            class: "form-input", r#type: "password",
                            placeholder: "••••••••",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }
                    button {
                        class: "btn btn-primary btn-full",
                        onclick: move |_| {
                            let mut m = msg.clone();
                            let e = email();
                            let p = password();
                            spawn(async move {
                                let client = crate::api::AuthApiClient::new("");
                                match client.login(&crate::api::LoginRequest {
                                    email: e, password: p, platform: "web".into(),
                                }).await {
                                    Ok(d) => {
                                        let s = crate::session::SessionManager::new();
                                        s.set_authenticated(&d.user_id, &d.access_token, &d.refresh_token);
                                        m.set(format!("Welcome, {}!", d.email.unwrap_or_default()));
                                    }
                                    Err(e) => m.set(e),
                                }
                            });
                        },
                        "Sign In"
                    }
                }
            }
        }
    }
}
