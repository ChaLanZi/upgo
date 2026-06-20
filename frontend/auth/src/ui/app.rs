use crate::api::{AuthApiClient, LoginRequest};
use crate::session::{AuthState, SessionManager};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Page {
    Login,
    Dashboard,
    Account,
    Files,
    Config,
}

#[component]
pub fn App() -> Element {
    let mut pg = use_signal(|| Page::Login);
    let mut auth = use_signal(|| AuthState::Unauthenticated);
    let mut msg = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);

    let uid = match auth() {
        AuthState::Authenticated { ref user_id, .. } => user_id.clone(),
        _ => String::new(),
    };
    let authed = matches!(auth(), AuthState::Authenticated { .. });

    rsx! {
        div { class: "app-root",
            if authed {
                nav { class: "navbar",
                    div { class: "nav-brand", onclick: move |_| pg.set(Page::Dashboard), "Upgo" }
                    div { class: "nav-links",
                        a { href: "#", onclick: move |_| pg.set(Page::Dashboard), "Home" }
                        a { href: "#", onclick: move |_| pg.set(Page::Files), "Files" }
                        a { href: "#", onclick: move |_| pg.set(Page::Account), "Account" }
                        a { href: "#", onclick: move |_| pg.set(Page::Config), "Config" }
                        button { onclick: move |_| { SessionManager::new().logout(); auth.set(AuthState::Unauthenticated); pg.set(Page::Login); }, "Logout" }
                    }
                }
                div { "User: {uid}" }
            }

            if !msg().is_empty() { div { "{msg}" } }

            match pg() {
                Page::Login => rsx! {
                    div { class: "auth-container",
                        div { class: "auth-card",
                            h1 { "Upgo" } p { "Sign in" }
                            div { class: "form-group",
                                label { "Email" }
                                input { r#type: "email", placeholder: "your@email.com",
                                    value: "{email}", oninput: move |e| email.set(e.value()) }
                            }
                            div { class: "form-group",
                                label { "Password" }
                                input { r#type: "password", placeholder: "••••••••",
                                    value: "{password}", oninput: move |e| password.set(e.value()) }
                            }
                            button { onclick: move |_| {
                                let e = email(); let p = password();
                                if e.is_empty() || p.is_empty() { msg.set("Required".into()); return; }
                                let mut a = auth.clone(); let mut m = msg.clone(); let mut p2 = pg.clone();
                                spawn(async move {
                                    let client = AuthApiClient::new("");
                                    match client.login(&LoginRequest { email: e, password: p, platform: "desktop".into() }).await {
                                        Ok(data) => {
                                            SessionManager::new().set_authenticated(&data.user_id, &data.access_token, &data.refresh_token);
                                            a.set(AuthState::Authenticated { user_id: data.user_id, access_token: data.access_token });
                                            m.set("Welcome!".into()); p2.set(Page::Dashboard);
                                        }
                                        Err(e) => m.set(e),
                                    }
                                });
                            }, "Sign In" }
                        }
                    }
                },
                Page::Dashboard => rsx! { h2 { "Dashboard" } p { "Welcome {uid}" } },
                Page::Account => rsx! { h2 { "Account" } p { "User ID: {uid}" } button { onclick: move |_| { SessionManager::new().logout(); auth.set(AuthState::Unauthenticated); pg.set(Page::Login); }, "Logout" } },
                Page::Files => rsx! { crate::ui::files::FileManager {} },
                Page::Config => rsx! { h2 { "Config" } p { "Settings" } },
            }
        }
    }
}
