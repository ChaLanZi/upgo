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
        div { class: "p-6 max-w-3xl mx-auto",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-semibold text-gray-800", "File Manager" }
                button { class: "px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors", onclick: refresh, "Refresh" }
            }
            if !status().is_empty() {
                div { class: "mb-4 px-4 py-3 bg-blue-50 text-blue-700 rounded-lg text-sm", "{status}" }
            }
            div { class: "space-y-2",
                for f in files() {
                    div { class: "flex items-center justify-between bg-white rounded-lg px-4 py-3 border border-gray-100 shadow-sm",
                        span { class: "text-sm text-gray-700 font-mono", "{f}" }
                        a { class: "text-xs text-blue-500 hover:text-blue-700", href: "/api/files/download/{f}", "Download" }
                    }
                }
                if files().is_empty() {
                    div { class: "text-center py-12 text-gray-400", "No files found. Click Refresh to load." }
                }
            }
        }
    }
}
