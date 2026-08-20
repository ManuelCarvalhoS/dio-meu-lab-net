use dioxus::prelude::*;
use meu_labnet_comum::{
    LiveResumo, PainelDto, PedidoPainelConfig, PedidoWidget, SessaoMeuLabNet, TipoWidget, Widget,
};

mod api;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const SESSAO_KEY: &str = "meu-labnet-sessao";

fn main() {
    dioxus::launch(App);
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn ler_sessao() -> Option<SessaoMeuLabNet> {
    let raw = storage()?.get_item(SESSAO_KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn ler_sessao() -> Option<SessaoMeuLabNet> {
    None
}

#[cfg(target_arch = "wasm32")]
fn guardar_sessao(s: &SessaoMeuLabNet) {
    if let (Some(st), Ok(json)) = (storage(), serde_json::to_string(s)) {
        let _ = st.set_item(SESSAO_KEY, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn guardar_sessao(_s: &SessaoMeuLabNet) {}

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

fn aplicar_tema_doc(tema: u8) {
    #[cfg(target_arch = "wasm32")]
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = doc.document_element() {
            let _ = el.set_attribute(
                "data-tema",
                if tema == 1 { "claro" } else { "escuro" },
            );
        }
    }
    let _ = tema;
}

fn reordenar_local(ids: &[u64], de: u64, para: u64) -> Option<Vec<u64>> {
    if de == para {
        return None;
    }
    let mut v = ids.to_vec();
    let i = v.iter().position(|&x| x == de)?;
    let j = v.iter().position(|&x| x == para)?;
    let item = v.remove(i);
    v.insert(j, item);
    Some(v)
}

fn url_atalho(slug: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let host = web_sys::window()
            .and_then(|w| w.location().hostname().ok())
            .unwrap_or_default();
        let prod = host.ends_with("labnetcol.pt");
        return match slug.trim().to_ascii_lowercase().as_str() {
            "labnetcol" => {
                if prod {
                    "https://labnetcol.pt".into()
                } else {
                    "http://localhost:8080".into()
                }
            }
            "lista" => {
                if prod {
                    "https://lista.labnetcol.pt".into()
                } else {
                    "http://localhost:8088".into()
                }
            }
            "agenda" | "superagenda" => {
                if prod {
                    "https://agenda.labnetcol.pt".into()
                } else {
                    "http://localhost:8091".into()
                }
            }
            "encripta" => {
                if prod {
                    "https://back1.labnetcol.pt".into()
                } else {
                    "http://localhost:8090".into()
                }
            }
            _ if slug.starts_with("http://") || slug.starts_with("https://") => slug.to_string(),
            _ => slug.to_string(),
        };
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        slug.to_string()
    }
}

#[component]
fn App() -> Element {
    let api_url = use_signal(api::api_url_padrao);
    let mut sessao = use_signal(ler_sessao);
    let mut sso_erro = use_signal(|| Option::<String>::None);
    let mut sso_a_processar = use_signal(|| ler_token_sso_url().is_some());
    let mut a_sair = use_signal(|| false);

    use_effect(move || {
        if let Some(tok) = ler_token_sso_url() {
            sso_a_processar.set(true);
            let url = api_url();
            spawn(async move {
                match api::sso_labnetcol(&url, &tok).await {
                    Ok(s) => {
                        guardar_sessao(&s);
                        sessao.set(Some(s));
                        limpar_url();
                        sso_erro.set(None);
                    }
                    Err(e) => sso_erro.set(Some(e.0)),
                }
                sso_a_processar.set(false);
            });
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Outfit:wght@300;500;600&family=Syne:wght@800&display=swap",
        }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "O Meu LabNet" }

        if sso_a_processar() {
            Porta { mensagem: "A validar sessão…" }
        } else if a_sair() {
            Porta { mensagem: "A sair…" }
        } else if let Some(e) = sso_erro() {
            PortaErro { mensagem: e }
        } else if let Some(s) = sessao() {
            Painel {
                api_url,
                sessao: s,
                on_sair: move |_| {
                    a_sair.set(true);
                    limpar_sessao();
                    ir_para(&url_logout_labnetcol());
                },
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
fn Painel(
    api_url: Signal<String>,
    sessao: SessaoMeuLabNet,
    on_sair: EventHandler<()>,
) -> Element {
    let token = use_signal(|| sessao.token.clone());
    let nome = sessao.nome.clone();
    let mut painel = use_signal(|| Option::<PainelDto>::None);
    let mut erro = use_signal(|| Option::<String>::None);
    let mut a_carregar = use_signal(|| true);
    let mut reload = use_signal(|| 0u32);

    let mut titulo_novo = use_signal(String::new);
    let mut conteudo_novo = use_signal(String::new);
    let mut tipo_novo = use_signal(|| TipoWidget::Link);

    use_effect(move || {
        let _ = reload();
        let url = api_url();
        let tok = token();
        a_carregar.set(true);
        spawn(async move {
            match api::obter_painel(&url, &tok).await {
                Ok(p) => {
                    aplicar_tema_doc(p.config.tema);
                    painel.set(Some(p));
                    erro.set(None);
                }
                Err(e) => erro.set(Some(e.0)),
            }
            a_carregar.set(false);
        });
    });

    let cols = painel()
        .map(|p| p.config.layout.clamp(1, 3))
        .unwrap_or(3);
    let tema = painel().map(|p| p.config.tema).unwrap_or(0);
    let classe_app = if tema == 1 {
        "painel-app tema-claro"
    } else {
        "painel-app"
    };

    rsx! {
        main { class: "{classe_app}",
            div { class: "bg-nodes", aria_hidden: true }
            div { class: "bg-glow bg-glow-cyan", aria_hidden: true }
            div { class: "bg-glow bg-glow-violet", aria_hidden: true }

            header { class: "painel-topo",
                div { class: "painel-esq",
                    div { class: "painel-brand",
                        span { class: "brand-small", "O Meu" }
                        span { class: "brand-main", "LabNet" }
                    }
                    p { class: "painel-saudacao", "Olá, {nome}" }
                }
                div { class: "painel-acoes",
                    button {
                        class: "tema-btn",
                        title: if tema == 1 { "Mudar para tema escuro" } else { "Mudar para tema claro" },
                        onclick: move |_| {
                            let novo = if tema == 1 { 0u8 } else { 1u8 };
                            aplicar_tema_doc(novo);
                            let url = api_url();
                            let tok = token();
                            spawn(async move {
                                let _ = api::actualizar_config(
                                    &url,
                                    &tok,
                                    &PedidoPainelConfig {
                                        tema: Some(novo),
                                        layout: None,
                                    },
                                )
                                .await;
                            });
                            reload.set(reload() + 1);
                        },
                        if tema == 1 { "Escuro" } else { "Claro" }
                    }
                    button { class: "sair-btn", onclick: move |_| on_sair.call(()), "Sair" }
                }
            }

            if a_carregar() {
                p { class: "painel-msg", "A carregar o teu painel…" }
            } else if let Some(e) = erro() {
                p { class: "painel-erro", "{e}" }
                button {
                    class: "porta-btn",
                    onclick: move |_| reload.set(reload() + 1),
                    "Tentar outra vez"
                }
            } else if let Some(p) = painel() {
                section {
                    class: "painel-grid",
                    style: "grid-template-columns: repeat({cols}, minmax(0, 1fr));",
                    ColunaWidgets {
                        key: "atalhos-{reload()}",
                        titulo: "Atalhos",
                        widgets: p.widgets.iter().filter(|w| w.tipo == TipoWidget::Atalho).cloned().collect(),
                        api_url,
                        token,
                        reload,
                    }
                    ColunaWidgets {
                        key: "links-{reload()}",
                        titulo: "Links",
                        widgets: p.widgets.iter().filter(|w| w.tipo == TipoWidget::Link).cloned().collect(),
                        api_url,
                        token,
                        reload,
                    }
                    ColunaWidgets {
                        key: "notas-{reload()}",
                        titulo: "Notas",
                        widgets: p.widgets.iter().filter(|w| w.tipo == TipoWidget::Nota).cloned().collect(),
                        api_url,
                        token,
                        reload,
                    }
                }

                section { class: "painel-add panel",
                    h2 { "Adicionar" }
                    div { class: "add-tabs",
                        button {
                            class: if tipo_novo() == TipoWidget::Link { "tab active" } else { "tab" },
                            onclick: move |_| tipo_novo.set(TipoWidget::Link),
                            "Link"
                        }
                        button {
                            class: if tipo_novo() == TipoWidget::Nota { "tab active" } else { "tab" },
                            onclick: move |_| tipo_novo.set(TipoWidget::Nota),
                            "Nota"
                        }
                    }
                    input {
                        class: "field",
                        placeholder: "Título",
                        value: "{titulo_novo}",
                        oninput: move |e| titulo_novo.set(e.value()),
                    }
                    if tipo_novo() == TipoWidget::Link {
                        input {
                            class: "field",
                            placeholder: "URL (https://…)",
                            value: "{conteudo_novo}",
                            oninput: move |e| conteudo_novo.set(e.value()),
                        }
                    } else {
                        textarea {
                            class: "field area",
                            placeholder: "Texto da nota",
                            value: "{conteudo_novo}",
                            oninput: move |e| conteudo_novo.set(e.value()),
                        }
                    }
                    button {
                        class: "btn-add",
                        onclick: move |_| {
                            let titulo = titulo_novo().trim().to_string();
                            let conteudo = conteudo_novo().trim().to_string();
                            if titulo.is_empty() { return; }
                            let url = api_url();
                            let tok = token();
                            let tipo = tipo_novo();
                            spawn(async move {
                                let pedido = PedidoWidget {
                                    tipo,
                                    titulo,
                                    conteudo,
                                    ordem: 99,
                                };
                                let _ = api::criar_widget(&url, &tok, &pedido).await;
                            });
                            titulo_novo.set(String::new());
                            conteudo_novo.set(String::new());
                            reload.set(reload() + 1);
                        },
                        "Guardar"
                    }
                }

                section { class: "painel-prefs panel",
                    h2 { "Preferências" }
                    div { class: "pref-row",
                        label { "Tema" }
                        div { class: "pref-btns",
                            button {
                                class: if tema == 0 { "pref pref-texto active" } else { "pref pref-texto" },
                                onclick: move |_| {
                                    aplicar_tema_doc(0);
                                    let url = api_url();
                                    let tok = token();
                                    spawn(async move {
                                        let _ = api::actualizar_config(
                                            &url,
                                            &tok,
                                            &PedidoPainelConfig {
                                                tema: Some(0),
                                                layout: None,
                                            },
                                        )
                                        .await;
                                    });
                                    reload.set(reload() + 1);
                                },
                                "Escuro"
                            }
                            button {
                                class: if tema == 1 { "pref pref-texto active" } else { "pref pref-texto" },
                                onclick: move |_| {
                                    aplicar_tema_doc(1);
                                    let url = api_url();
                                    let tok = token();
                                    spawn(async move {
                                        let _ = api::actualizar_config(
                                            &url,
                                            &tok,
                                            &PedidoPainelConfig {
                                                tema: Some(1),
                                                layout: None,
                                            },
                                        )
                                        .await;
                                    });
                                    reload.set(reload() + 1);
                                },
                                "Claro"
                            }
                        }
                    }
                    div { class: "pref-row",
                        label { "Colunas" }
                        div { class: "pref-btns",
                            for n in 1u8..=3 {
                                button {
                                    class: if p.config.layout == n { "pref active" } else { "pref" },
                                    onclick: {
                                        move |_| {
                                            let url = api_url();
                                            let tok = token();
                                            let colunas = n;
                                            spawn(async move {
                                                let _ = api::actualizar_config(
                                                    &url,
                                                    &tok,
                                                    &PedidoPainelConfig {
                                                        tema: None,
                                                        layout: Some(colunas),
                                                    },
                                                ).await;
                                            });
                                            reload.set(reload() + 1);
                                        }
                                    },
                                    "{n}"
                                }
                            }
                        }
                    }
                    p { class: "pref-dica", "Arrasta os cartões (ou usa ↑↓) para reordenar." }
                }
            }
        }
    }
}

fn persistir_ordem(
    api_url: Signal<String>,
    token: Signal<String>,
    mut reload: Signal<u32>,
    ids: Vec<u64>,
) {
    let url = api_url();
    let tok = token();
    spawn(async move {
        let _ = api::reordenar_widgets(&url, &tok, &ids).await;
        reload.set(reload() + 1);
    });
}

#[component]
fn ColunaWidgets(
    titulo: &'static str,
    widgets: Vec<Widget>,
    api_url: Signal<String>,
    token: Signal<String>,
    mut reload: Signal<u32>,
) -> Element {
    let mut lista = use_signal(|| widgets.clone());
    let mut arrastando = use_signal(|| Option::<u64>::None);
    let mut alvo = use_signal(|| Option::<u64>::None);

    rsx! {
        div { class: "coluna panel",
            h2 { "{titulo}" }
            if lista().is_empty() {
                p { class: "vazio", "Ainda vazio." }
            }
            for (idx, w) in lista().into_iter().enumerate() {
                {
                    let id = w.n_reg;
                    let a_arrastar = arrastando() == Some(id);
                    let e_alvo = alvo() == Some(id) && arrastando().is_some_and(|a| a != id);
                    let classe = if a_arrastar {
                        "widget-card a-arrastar"
                    } else if e_alvo {
                        "widget-card drop-alvo"
                    } else {
                        "widget-card"
                    };
                    let n = lista().len();
                    rsx! {
                        article {
                            class: "{classe}",
                            key: "{id}",
                            draggable: "true",
                            ondragstart: move |_| {
                                arrastando.set(Some(id));
                            },
                            ondragover: move |ev| {
                                ev.prevent_default();
                                if arrastando().is_some_and(|a| a != id) {
                                    alvo.set(Some(id));
                                }
                            },
                            ondragleave: move |_| {
                                if alvo() == Some(id) {
                                    alvo.set(None);
                                }
                            },
                            ondrop: move |ev| {
                                ev.prevent_default();
                                let Some(de) = arrastando() else { return; };
                                let ids: Vec<u64> = lista().iter().map(|x| x.n_reg).collect();
                                if let Some(nova) = reordenar_local(&ids, de, id) {
                                    let mut actual = lista();
                                    actual.sort_by_key(|w| {
                                        nova.iter().position(|&x| x == w.n_reg).unwrap_or(999)
                                    });
                                    lista.set(actual);
                                    persistir_ordem(api_url, token, reload, nova);
                                }
                                arrastando.set(None);
                                alvo.set(None);
                            },
                            ondragend: move |_| {
                                arrastando.set(None);
                                alvo.set(None);
                            },
                            div { class: "widget-head",
                                span { class: "drag-handle", title: "Arrastar para reordenar", "⠿" }
                                strong { "{w.titulo}" }
                                div { class: "widget-acoes",
                                    button {
                                        class: "widget-ord",
                                        title: "Subir",
                                        disabled: idx == 0,
                                        onclick: move |_| {
                                            if idx == 0 { return; }
                                            let ids: Vec<u64> = lista().iter().map(|x| x.n_reg).collect();
                                            let acima = ids[idx - 1];
                                            if let Some(nova) = reordenar_local(&ids, id, acima) {
                                                let mut actual = lista();
                                                actual.sort_by_key(|w| {
                                                    nova.iter().position(|&x| x == w.n_reg).unwrap_or(999)
                                                });
                                                lista.set(actual);
                                                persistir_ordem(api_url, token, reload, nova);
                                            }
                                        },
                                        "↑"
                                    }
                                    button {
                                        class: "widget-ord",
                                        title: "Descer",
                                        disabled: idx + 1 >= n,
                                        onclick: move |_| {
                                            if idx + 1 >= n { return; }
                                            let ids: Vec<u64> = lista().iter().map(|x| x.n_reg).collect();
                                            let abaixo = ids[idx + 1];
                                            if let Some(nova) = reordenar_local(&ids, id, abaixo) {
                                                let mut actual = lista();
                                                actual.sort_by_key(|w| {
                                                    nova.iter().position(|&x| x == w.n_reg).unwrap_or(999)
                                                });
                                                lista.set(actual);
                                                persistir_ordem(api_url, token, reload, nova);
                                            }
                                        },
                                        "↓"
                                    }
                                    button {
                                        class: "widget-del",
                                        title: "Remover",
                                        onclick: move |_| {
                                            let url = api_url();
                                            let tok = token();
                                            spawn(async move {
                                                let _ = api::remover_widget(&url, &tok, id).await;
                                            });
                                            reload.set(reload() + 1);
                                        },
                                        "×"
                                    }
                                }
                            }
                            match w.tipo {
                                TipoWidget::Link => rsx! {
                                    a {
                                        class: "widget-link",
                                        href: "{w.conteudo}",
                                        target: "_blank",
                                        rel: "noopener",
                                        "{w.conteudo}"
                                    }
                                },
                                TipoWidget::Atalho => {
                                    let href = url_atalho(&w.conteudo);
                                    let slug = w.conteudo.clone();
                                    rsx! {
                                        a {
                                            class: "widget-link atalho",
                                            href: "{href}",
                                            target: "_blank",
                                            rel: "noopener",
                                            "Abrir →"
                                        }
                                        LinhaLive { api_url, token, slug }
                                    }
                                },
                                TipoWidget::Nota => rsx! {
                                    p { class: "widget-nota", "{w.conteudo}" }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LinhaLive(api_url: Signal<String>, token: Signal<String>, slug: String) -> Element {
    let mut resumo = use_signal(|| Option::<LiveResumo>::None);
    let mut a_carregar = use_signal(|| true);

    use_effect({
        let slug = slug.clone();
        move || {
            let url = api_url();
            let tok = token();
            let slug = slug.clone();
            a_carregar.set(true);
            spawn(async move {
                match api::live_resumo(&url, &tok, &slug).await {
                    Ok(r) => resumo.set(Some(r)),
                    Err(_) => resumo.set(Some(LiveResumo::aviso(&slug, "—"))),
                }
                a_carregar.set(false);
            });
        }
    });

    if a_carregar() {
        return rsx! { p { class: "live-linha a-carregar", "A actualizar…" } };
    }
    match resumo() {
        Some(r) => {
            let classe = if r.ok {
                "live-linha ok"
            } else {
                "live-linha aviso"
            };
            rsx! { p { class: "{classe}", "{r.linha}" } }
        }
        None => rsx! {},
    }
}
