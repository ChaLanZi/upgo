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
    let backend = std::env::var("UPGO_BACKEND").unwrap_or_else(|_| "http://localhost:80".into());
    let msg_cls = if msg().contains('e') || msg().contains('o') {
        "bg-red-50 text-red-700"
    } else {
        "bg-blue-50 text-blue-700"
    };

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            if authed {
                nav { class: "bg-white shadow-sm border-b border-gray-200 px-6 py-3 flex items-center justify-between",
                    div { class: "flex items-center gap-8",
                        span { class: "text-xl font-bold text-blue-500 cursor-pointer", onclick: move |_| pg.set(Page::Dashboard), "Upgo" }
                        div { class: "flex gap-4 text-sm",
                            a { href: "#", class: "text-gray-600 hover:text-blue-500", onclick: move |_| pg.set(Page::Dashboard), "Home" }
                            a { href: "#", class: "text-gray-600 hover:text-blue-500", onclick: move |_| pg.set(Page::Files), "Files" }
                            a { href: "#", class: "text-gray-600 hover:text-blue-500", onclick: move |_| pg.set(Page::Account), "Account" }
                            a { href: "#", class: "text-gray-600 hover:text-blue-500", onclick: move |_| pg.set(Page::Config), "Config" }
                        }
                    }
                    div { class: "flex items-center gap-4",
                        span { class: "text-xs text-gray-400", "User: {uid}" }
                        button { class: "px-3 py-1.5 text-xs bg-red-50 text-red-600 rounded-md hover:bg-red-100", onclick: move |_| { SessionManager::new().logout(); auth.set(AuthState::Unauthenticated); pg.set(Page::Login); }, "Logout" }
                    }
                }
            }
            if !msg().is_empty() {
                div { class: "mx-6 mt-4 px-4 py-3 rounded-lg text-sm {msg_cls}", "{msg}" }
            }
            match pg() {
                Page::Login => rsx! {
                    div { class: "flex items-center justify-center min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100",
                        div { class: "bg-white rounded-2xl shadow-lg p-8 w-full max-w-sm mx-4",
                            h1 { class: "text-3xl font-bold text-center text-blue-500 mb-1", "Upgo" }
                            p { class: "text-sm text-gray-400 text-center mb-6", "Sign in" }
                            div { class: "mb-4",
                                label { class: "block text-xs font-medium text-gray-500 mb-1.5", "Email" }
                                input { r#type: "email", placeholder: "your@email.com", class: "w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400",
                                    value: "{email}", oninput: move |e| email.set(e.value()) }
                            }
                            div { class: "mb-6",
                                label { class: "block text-xs font-medium text-gray-500 mb-1.5", "Password" }
                                input { r#type: "password", placeholder: "••••••••", class: "w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400",
                                    value: "{password}", oninput: move |e| password.set(e.value()) }
                            }
                            button { class: "w-full py-2.5 bg-blue-500 text-white rounded-lg text-sm font-medium hover:bg-blue-600 transition-colors", onclick: move |_| {
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
                Page::Dashboard => rsx! {
                    div { class: "p-6 max-w-3xl mx-auto",
                        h2 { class: "text-xl font-semibold text-gray-800 mb-4", "Dashboard" }
                        p { class: "text-gray-600", "Welcome, {uid}" }
                        div { class: "grid grid-cols-3 gap-4 mt-6",
                            div { class: "bg-white rounded-xl p-5 shadow-sm border border-gray-100 cursor-pointer hover:shadow-md", onclick: move |_| pg.set(Page::Files), div { class: "text-2xl mb-2", "📁" } h3 { class: "font-medium text-gray-800", "Files" } p { class: "text-xs text-gray-400 mt-1", "Manage files" } }
                            div { class: "bg-white rounded-xl p-5 shadow-sm border border-gray-100 cursor-pointer hover:shadow-md", onclick: move |_| pg.set(Page::Account), div { class: "text-2xl mb-2", "👤" } h3 { class: "font-medium text-gray-800", "Account" } p { class: "text-xs text-gray-400 mt-1", "Profile & settings" } }
                            div { class: "bg-white rounded-xl p-5 shadow-sm border border-gray-100 cursor-pointer hover:shadow-md", onclick: move |_| pg.set(Page::Config), div { class: "text-2xl mb-2", "⚙️" } h3 { class: "font-medium text-gray-800", "Config" } p { class: "text-xs text-gray-400 mt-1", "Preferences" } }
                        }
                    }
                },
                Page::Account => rsx! {
                    div { class: "p-6 max-w-xl mx-auto",
                        h2 { class: "text-xl font-semibold text-gray-800 mb-4", "Account" }
                        div { class: "bg-white rounded-xl p-5 shadow-sm border border-gray-100 mb-4",
                            h3 { class: "font-medium text-gray-800 mb-2", "Profile" }
                            p { class: "text-sm text-gray-500", "User ID: {uid}" }
                        }
                        div { class: "bg-white rounded-xl p-5 shadow-sm border border-gray-100",
                            h3 { class: "font-medium text-gray-800 mb-2", "Sessions" }
                            p { class: "text-sm text-gray-500 mb-3", "Manage active sessions." }
                            button { class: "px-4 py-2 text-sm bg-red-50 text-red-600 rounded-lg hover:bg-red-100", onclick: move |_| { SessionManager::new().logout(); auth.set(AuthState::Unauthenticated); pg.set(Page::Login); }, "Logout All" }
                        }
                    }
                },
                Page::Files => rsx! { crate::ui::files::FileManager {} },
                Page::Config => rsx! {
                    div { class: "p-6 max-w-xl mx-auto",
                        h2 { class: "text-xl font-semibold text-gray-800 mb-4", "Configuration" }
                        div { class: "bg-white rounded-xl p-5 shadow-sm border border-gray-100",
                            h3 { class: "font-medium text-gray-800 mb-3", "API Backend" }
                            input { class: "w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-400", value: "{backend}" }
                            p { class: "text-xs text-gray-400 mt-2", "Gateway backend URL." }
                        }
                    }
                },
            }
        }
    }
}
