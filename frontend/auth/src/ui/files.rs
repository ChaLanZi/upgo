use dioxus::prelude::*;

use reqwest::Client;

#[component]
pub fn FileManager() -> Element {
    let mut files = use_signal(Vec::<String>::new);
    let mut status = use_signal(String::new);

    let refresh = move |_evt: dioxus::events::MouseEvent| {
        let mut f = files.clone();
        let mut s = status.clone();
        spawn(async move {
            match Client::new().get("/api/files/list").send().await {
                Ok(r) => {
                    if let Ok(data) = r.json::<serde_json::Value>().await {
                        let names: Vec<String> = data["files"]
                            .as_array()
                            .map(|a| {
                                a.iter()
                                    .filter_map(|x| x["key"].as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        f.set(names);
                        s.set(format!("Loaded {} files", f.peek().len()));
                    }
                }
                Err(e) => s.set(format!("Error: {}", e)),
            }
        });
    };

    rsx! {
        div { class: "files-container",
            h2 { "File Manager" }
            button { onclick: refresh, "Refresh" }
            if !status().is_empty() { div { "{status}" } }
            div { class: "file-list",
                for f in files() {
                    div { class: "file-item",
                        span { "{f}" }
                        a { href: "/api/files/download/{f}", "Download" }
                    }
                }
                if files().is_empty() {
                    p { "No files." }
                }
            }
        }
    }
}
