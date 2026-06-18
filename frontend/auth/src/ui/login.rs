use dioxus::prelude::*;

#[component]
pub fn LoginPage(
    on_login: EventHandler<(String, String)>,
    on_register: EventHandler<()>,
    error: String,
) -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);

    rsx! {
        div { class: "auth-container",
            div { class: "auth-card",
                h1 { class: "auth-title", "Upgo" }
                p { class: "auth-subtitle", "Sign in to your account" }

                if !error.is_empty() {
                    div { class: "alert alert-error", "{error}" }
                }

                div { class: "form-group",
                    label { "Email" }
                    input {
                        class: "form-input",
                        r#type: "email",
                        placeholder: "your@email.com",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { "Password" }
                    input {
                        class: "form-input",
                        r#type: "password",
                        placeholder: "••••••••",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }
                button {
                    class: "btn btn-primary btn-full",
                    onclick: move |_| on_login.call((email(), password())),
                    "Sign In"
                }
                p { class: "auth-footer",
                    "Don't have an account? "
                    a { href: "#", onclick: move |_| on_register.call(()), "Register" }
                }
            }
        }
    }
}
