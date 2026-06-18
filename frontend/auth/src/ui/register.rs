use dioxus::prelude::*;

#[component]
pub fn RegisterPage(
    on_register: EventHandler<(String, String, String)>,
    on_verify: EventHandler<(String, String)>,
    on_login: EventHandler<()>,
    error: String,
) -> Element {
    let mut step = use_signal(|| 1u8);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut nickname = use_signal(String::new);
    let mut code = use_signal(String::new);

    rsx! {
        div { class: "auth-container",
            div { class: "auth-card",
                h1 { class: "auth-title", "Create Account" }

                if !error.is_empty() {
                    div { class: "alert alert-info", "{error}" }
                }

                // Step 1: Register
                if step() == 1 {
                    div { class: "form-group",
                        label { "Email" }
                        input { class: "form-input", r#type: "email", placeholder: "your@email.com",
                            value: "{email}", oninput: move |e| email.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Nickname" }
                        input { class: "form-input", placeholder: "Display name",
                            value: "{nickname}", oninput: move |e| nickname.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Password" }
                        input { class: "form-input", r#type: "password", placeholder: "Min 8 characters",
                            value: "{password}", oninput: move |e| password.set(e.value()) }
                    }
                    button { class: "btn btn-primary btn-full",
                        onclick: move |_| { on_register.call((email(), password(), nickname())); step.set(2); },
                        "Send Verification Code" }
                }

                // Step 2: Verify Email
                if step() == 2 {
                    div { class: "form-group",
                        label { "Verification Code" }
                        input { class: "form-input", placeholder: "6-digit code",
                            value: "{code}", oninput: move |e| code.set(e.value()) }
                    }
                    button { class: "btn btn-primary btn-full",
                        onclick: move |_| on_verify.call((email(), code())),
                        "Verify & Complete" }
                }

                p { class: "auth-footer",
                    "Already have an account? "
                    a { href: "#", onclick: move |_| on_login.call(()), "Sign In" }
                }
            }
        }
    }
}
