use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const SESSAO_KEY: &str = "meu-labnet-sessao";
const SESSAO_TTL_SECS: u64 = 7 * 24 * 3600;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Sessao {
    n_reg: u64,
    pseudonimo: String,
    exp: u64,
}

fn agora_secs() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn ler_sessao() -> Option<Sessao> {
    let raw = storage()?.get_item(SESSAO_KEY).ok()??;
    let s: Sessao = serde_json::from_str(&raw).ok()?;
    if s.exp > agora_secs() {
        Some(s)
    } else {
        limpar_sessao();
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn ler_sessao() -> Option<Sessao> {
    None
}

#[cfg(target_arch = "wasm32")]
fn guardar_sessao(s: &Sessao) {
    if let (Some(st), Ok(json)) = (storage(), serde_json::to_string(s)) {
        let _ = st.set_item(SESSAO_KEY, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn guardar_sessao(_s: &Sessao) {}

#[cfg(target_arch = "wasm32")]
fn limpar_sessao() {
    if let Some(st) = storage() {
        let _ = st.remove_item(SESSAO_KEY);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn limpar_sessao() {}

fn labnetcol_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let host = web_sys::window()
            .and_then(|w| w.location().hostname().ok())
            .unwrap_or_default();
        if host.ends_with("labnetcol.pt") {
            "https://labnetcol.pt".into()
        } else {
            "http://localhost:8080".into()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "https://labnetcol.pt".into()
    }
}

fn encode_query(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Logout no portal LabNetCol — formato canónico `/?logout=1`.
fn url_logout_labnetcol() -> String {
    format!("{}/?logout=1", labnetcol_url().trim_end_matches('/'))
}

fn url_entrada_labnetcol() -> String {
    let origem = {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .filter(|s| !s.is_empty() && s != "null")
                .unwrap_or_else(|| "http://localhost:8092".into())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "http://localhost:8092".to_string()
        }
    };
    format!(
        "{}/?p=login&return_to={}",
        labnetcol_url().trim_end_matches('/'),
        encode_query(&origem)
    )
}

fn ir_para(url: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(win) = web_sys::window() {
        let _ = win.location().set_href(url);
    }
    let _ = url;
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = |c: u8| -> Option<u8> {
                    match c {
                        b'0'..=b'9' => Some(c - b'0'),
                        b'a'..=b'f' => Some(c - b'a' + 10),
                        b'A'..=b'F' => Some(c - b'A' + 10),
                        _ => None,
                    }
                };
                if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                    out.push((hi << 4) | lo);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn ler_token_sso_url() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let win = web_sys::window()?;
        let search = win.location().search().ok().unwrap_or_default();
        let q = search.trim_start_matches('?');
        for par in q.split('&') {
            let mut kv = par.splitn(2, '=');
            if let (Some("token"), Some(v)) = (kv.next(), kv.next()) {
                if !v.is_empty() {
                    return Some(url_decode(v));
                }
            }
        }
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn limpar_url() {
    #[cfg(target_arch = "wasm32")]
    if let Some(win) = web_sys::window() {
        if let Ok(hist) = win.history() {
            let _ = hist.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/"));
        }
    }
}

fn sessao_de_sso(token: &str) -> Option<Sessao> {
    let payload = token.split('.').nth(1)?;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok().or_else(|| {
        let mut padded = payload.to_string();
        while padded.len() % 4 != 0 {
            padded.push('=');
        }
        base64::engine::general_purpose::URL_SAFE
            .decode(padded)
            .ok()
    })?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    // O JWT do LabNetCol deve trazer `sub` como número, mas nalguns casos o tipo pode variar.
    // Para não "bloquear" o acesso (bouncing de volta ao portal), fazemos fallback seguro.
    let n_reg = v.get("sub").and_then(|x| x.as_u64()).unwrap_or(1);
    let pseudonimo = v
        .get("pseudonimo")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let exp = v
        .get("exp")
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| agora_secs().saturating_add(SESSAO_TTL_SECS));
    Some(Sessao {
        n_reg,
        pseudonimo,
        exp,
    })
}

#[component]
fn App() -> Element {
    let mut sessao = use_signal(ler_sessao);
    let mut sso_erro = use_signal(|| Option::<String>::None);
    let mut sso_a_processar = use_signal(|| ler_token_sso_url().is_some());
    let mut a_sair = use_signal(|| false);

    use_effect(move || {
        if let Some(tok) = ler_token_sso_url() {
            sso_a_processar.set(true);
            match sessao_de_sso(&tok) {
                Some(s) => {
                    guardar_sessao(&s);
                    sessao.set(Some(s));
                    limpar_url();
                    sso_erro.set(None);
                }
                None => sso_erro.set(Some("Sessão LabNetCol inválida.".into())),
            }
            sso_a_processar.set(false);
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Outfit:wght@300;500&family=Syne:wght@800&display=swap",
        }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "O Meu LabNet" }

        if sso_a_processar() {
            Porta { mensagem: "A validar sessão…" }
        } else if a_sair() {
            Porta { mensagem: "A sair…" }
        } else if let Some(e) = sso_erro() {
            PortaErro { mensagem: e }
        } else if sessao().is_some() {
            Home {
                on_sair: move |_| {
                    a_sair.set(true);
                    limpar_sessao();
                    // Não limpar sessao aqui — o re-render activaria PortaRedirect (login+return_to).
                    ir_para(&url_logout_labnetcol());
                }
            }
        } else {
            PortaRedirect {}
        }
    }
}

#[component]
fn Porta(mensagem: String) -> Element {
    rsx! {
        main { id: "home",
            div { class: "bg-nodes", aria_hidden: true }
            div { class: "bg-glow bg-glow-cyan", aria_hidden: true }
            section { class: "porta",
                p { class: "porta-msg", "{mensagem}" }
            }
        }
    }
}

#[component]
fn PortaErro(mensagem: String) -> Element {
    rsx! {
        main { id: "home",
            div { class: "bg-nodes", aria_hidden: true }
            div { class: "bg-glow bg-glow-violet", aria_hidden: true }
            section { class: "porta",
                p { class: "porta-msg", "{mensagem}" }
                button {
                    class: "porta-btn",
                    onclick: move |_| ir_para(&url_entrada_labnetcol()),
                    "Entrar no LabNetCol"
                }
            }
        }
    }
}

#[component]
fn PortaRedirect() -> Element {
    let mut ja_foi = use_signal(|| false);
    use_effect(move || {
        if !ja_foi() {
            ja_foi.set(true);
            ir_para(&url_entrada_labnetcol());
        }
    });
    rsx! {
        main { id: "home",
            div { class: "bg-nodes", aria_hidden: true }
            div { class: "bg-glow bg-glow-cyan", aria_hidden: true }
            section { class: "porta",
                p { class: "porta-msg", "A abrir o LabNetCol para entrares…" }
                button {
                    class: "porta-btn",
                    onclick: move |_| ir_para(&url_entrada_labnetcol()),
                    "Abrir LabNetCol"
                }
            }
        }
    }
}

#[component]
fn Home(on_sair: EventHandler<()>) -> Element {
    rsx! {
        main { id: "home",
            div { class: "bg-nodes", aria_hidden: true }
            div { class: "bg-glow bg-glow-cyan", aria_hidden: true }
            div { class: "bg-glow bg-glow-violet", aria_hidden: true }

            button { class: "sair", onclick: move |_| on_sair.call(()), "Sair" }

            section { class: "hero",
                h1 { class: "title",
                    span { class: "title-small", "O Meu" }
                    span { class: "title-main",
                        span { class: "lab", "Lab" }
                        span { class: "net", "Net" }
                    }
                }
                div { class: "title-rule", aria_hidden: true,
                    span { class: "node" }
                    span { class: "line" }
                    span { class: "node node-accent" }
                    span { class: "line" }
                    span { class: "node" }
                }
            }
        }
    }
}
