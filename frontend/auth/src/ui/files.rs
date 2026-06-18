//! File management UI using the FRS API.

use dioxus::prelude::*;

#[component]
pub fn FileManager() -> Element {
    let mut files = use_signal(Vec::<String>::new);
    let mut key = use_signal(String::new);
    let mut status = use_signal(String::new);

    rsx! {
        div { class: "files-container",
            h2 { "File Manager" }

            div { class: "file-upload",
                input { class: "form-input", placeholder: "File key (optional)",
                    value: "{key}", oninput: move |e| key.set(e.value()) }
                button { class: "btn btn-primary", onclick: {
                    let mut files = files.clone();
                    let mut status = status.clone();
                    move |_| {
                        let mut files = files.clone();
                        let mut status = status.clone();
                        spawn(async move {
                            let resp = reqwest::get("/api/files/list").await;
                            match resp {
                                Ok(r) => {
                                    if let Ok(data) = r.json::<serde_json::Value>().await {
                                        let names: Vec<String> = data["files"]
                                            .as_array()
                                            .map(|arr| arr.iter().filter_map(|f| f["key"].as_str().map(String::from)).collect())
                                            .unwrap_or_default();
                                        let count = names.len();
                                        files.set(names);
                                        status.set(format!("Loaded {} files", count));
                                    }
                                }
                                Err(e) => status.set(format!("Load failed: {}", e)),
                            }
                        });
                    }
                }, "Refresh" }
            }

            if !status().is_empty() {
                div { class: "alert alert-info", "{status}" }
            }

            div { class: "file-list",
                h3 { "Files ({files().len()})" }
                for f in files() {
                    div { class: "file-item",
                        span { "{f}" }
                        a { class: "btn btn-sm", href: "/api/files/download/{f}", "Download" }
                    }
                }
                if files().is_empty() {
                    p { class: "empty-msg", "No files. Refresh to load." }
                }
            }
        }
    }
}
