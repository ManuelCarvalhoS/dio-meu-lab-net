use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use meu_labnet_comum::{
    agora_unix, PainelConfig, PainelDto, PedidoPainelConfig, TipoWidget, Widget,
};
use mcs_bd2::{bd_alterar, bd_gravar, bd_listar_por_uid};

use crate::entidades::{PainelConfigReg, WidgetReg};

use crate::estado::AppState;
use crate::rotas::auth_rota::verificar_token;

macro_rules! exige {
    ($headers:expr, $state:expr) => {
        match verificar_token(&$headers, &$state.jwt_secret) {
            Some(s) => s,
            None => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"erro":"Não autenticado. Entra pelo LabNetCol."})),
                )
                    .into_response()
            }
        }
    };
}

async fn obter_ou_criar_config(state: &AppState, uid: u64) -> PainelConfig {
    let bd = match state.bd.get("painel") {
        Some(b) => b.clone(),
        None => return PainelConfig::padrao(uid),
    };
    if let Ok(regs) = bd_listar_por_uid::<PainelConfigReg>(bd, uid).await {
        if let Some((n, r)) = regs.into_iter().next() {
            return r.to_config(n);
        }
    }
    let mut cfg = PainelConfig::padrao(uid);
    if let Some(bd) = state.bd.get("painel") {
        if let Ok(n) = bd_gravar(bd.clone(), PainelConfigReg::from_config(&cfg)).await {
            cfg.n_reg = n;
        }
    }
    cfg
}

async fn listar_widgets(state: &AppState, uid: u64) -> Vec<Widget> {
    let Some(bd) = state.bd.get("widget") else {
        return Vec::new();
    };
    let Ok(regs) = bd_listar_por_uid::<WidgetReg>(bd.clone(), uid).await else {
        return Vec::new();
    };
    let mut v: Vec<Widget> = regs
        .into_iter()
        .map(|(n, r)| r.to_widget(n))
        .filter(|w| w.activo)
        .collect();
    v.sort_by_key(|w| (w.ordem, w.n_reg));
    v
}

async fn garantir_atalhos_iniciais(state: &AppState, uid: u64) -> Vec<Widget> {
    let mut widgets = listar_widgets(state, uid).await;
    let Some(bd) = state.bd.get("widget") else {
        return widgets;
    };

    let desejados: &[(&str, &str)] = &[
        ("LabNetCol", "labnetcol"),
        ("Lista", "lista"),
        ("Agenda", "agenda"),
        ("Encripta", "encripta"),
    ];

    if widgets.is_empty() {
        for (i, (titulo, slug)) in desejados.iter().enumerate() {
            let w = Widget::novo(uid, TipoWidget::Atalho, titulo, slug, i as u16);
            if let Ok(n) = bd_gravar(bd.clone(), WidgetReg::from_widget(&w)).await {
                let mut guardado = w;
                guardado.n_reg = n;
                widgets.push(guardado);
            }
        }
        return widgets;
    }

    // Utilizadores antigos: acrescenta Agenda se faltar
    let tem_agenda = widgets
        .iter()
        .any(|w| w.tipo == TipoWidget::Atalho && w.conteudo.eq_ignore_ascii_case("agenda"));
    if !tem_agenda {
        let ordem = widgets
            .iter()
            .filter(|w| w.tipo == TipoWidget::Atalho)
            .map(|w| w.ordem)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let w = Widget::novo(uid, TipoWidget::Atalho, "Agenda", "agenda", ordem);
        if let Ok(n) = bd_gravar(bd.clone(), WidgetReg::from_widget(&w)).await {
            let mut guardado = w;
            guardado.n_reg = n;
            widgets.push(guardado);
            widgets.sort_by_key(|w| (w.ordem, w.n_reg));
        }
    }
    widgets
}

pub async fn obter_painel(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    let uid = sessao.labnetcol_id;
    let config = obter_ou_criar_config(&state, uid).await;
    let widgets = garantir_atalhos_iniciais(&state, uid).await;
    Json(PainelDto { config, widgets }).into_response()
}

pub async fn actualizar_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(pedido): Json<PedidoPainelConfig>,
) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    let uid = sessao.labnetcol_id;
    let Some(bd) = state.bd.get("painel") else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"erro":"Serviço indisponível"})),
        )
            .into_response();
    };
    let mut cfg = obter_ou_criar_config(&state, uid).await;
    if let Some(t) = pedido.tema {
        cfg.tema = t;
    }
    if let Some(l) = pedido.layout {
        cfg.layout = l.clamp(1, 3);
    }
    cfg.actualizado_em = agora_unix();
    let reg = PainelConfigReg::from_config(&cfg);
    if cfg.n_reg == 0 {
        match bd_gravar(bd.clone(), reg).await {
            Ok(n) => {
                cfg.n_reg = n;
                Json(cfg).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"erro": e.to_string()})),
            )
                .into_response(),
        }
    } else {
        match bd_alterar(bd.clone(), reg, cfg.n_reg).await {
            Ok(_) => Json(cfg).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"erro": e.to_string()})),
            )
                .into_response(),
        }
    }
}
