use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use meu_labnet_comum::{agora_unix, PedidoReordenar, PedidoWidget, PedidoWidgetPatch, Widget};
use mcs_bd2::{bd_alterar, bd_gravar, bd_ler_dados, bd_remover};

use crate::entidades::WidgetReg;

use crate::estado::AppState;
use crate::rotas::auth_rota::verificar_token;

macro_rules! exige {
    ($headers:expr, $state:expr) => {
        match verificar_token(&$headers, &$state.jwt_secret) {
            Some(s) => s,
            None => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"erro":"Não autenticado."})),
                )
                    .into_response()
            }
        }
    };
}

async fn widget_do_dono(
    state: &AppState,
    id: u64,
    dono: u64,
) -> Result<WidgetReg, (StatusCode, Json<serde_json::Value>)> {
    let bd = state.bd.get("widget").cloned().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"erro":"Serviço indisponível"})),
    ))?;
    let Some(reg) = bd_ler_dados::<WidgetReg>(bd, id)
        .await
        .ok()
        .flatten()
    else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"erro":"Widget não encontrado."})),
        ));
    };
    if reg.utilizador != dono {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"erro":"Este widget não é teu."})),
        ));
    }
    Ok(reg)
}

pub async fn criar_widget(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(pedido): Json<PedidoWidget>,
) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    let titulo = pedido.titulo.trim();
    if titulo.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"erro":"Indica um título."})),
        )
            .into_response();
    }
    let Some(bd) = state.bd.get("widget") else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"erro":"Serviço indisponível"})),
        )
            .into_response();
    };
    let w = Widget::novo(
        sessao.labnetcol_id,
        pedido.tipo,
        titulo,
        &pedido.conteudo,
        pedido.ordem,
    );
    match bd_gravar(bd.clone(), WidgetReg::from_widget(&w)).await {
        Ok(n) => {
            let mut out = w;
            out.n_reg = n;
            Json(out).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"erro": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn editar_widget(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(pedido): Json<PedidoWidgetPatch>,
) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    let reg = match widget_do_dono(&state, id, sessao.labnetcol_id).await {
        Ok(r) => r,
        Err(r) => return r.into_response(),
    };
    let mut w = reg.to_widget(id);
    if let Some(t) = pedido.titulo {
        let t = t.trim().to_string();
        if t.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"erro":"Título inválido."})),
            )
                .into_response();
        }
        w.titulo = t;
    }
    if let Some(c) = pedido.conteudo {
        w.conteudo = c.trim().to_string();
    }
    if let Some(o) = pedido.ordem {
        w.ordem = o;
    }
    if let Some(a) = pedido.activo {
        w.activo = a;
    }
    w.actualizado_em = agora_unix();
    let Some(bd) = state.bd.get("widget") else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"erro":"Serviço indisponível"})),
        )
            .into_response();
    };
    match bd_alterar(bd.clone(), WidgetReg::from_widget(&w), id).await {
        Ok(_) => Json(w).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"erro": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn reordenar_widgets(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(pedido): Json<PedidoReordenar>,
) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    if pedido.ids.is_empty() {
        return StatusCode::NO_CONTENT.into_response();
    }
    if pedido.ids.len() > 200 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"erro":"Demasiados widgets."})),
        )
            .into_response();
    }
    let Some(bd) = state.bd.get("widget") else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"erro":"Serviço indisponível"})),
        )
            .into_response();
    };
    let mut tipo_ref: Option<u8> = None;
    for (i, &id) in pedido.ids.iter().enumerate() {
        let reg = match widget_do_dono(&state, id, sessao.labnetcol_id).await {
            Ok(r) => r,
            Err(r) => return r.into_response(),
        };
        if let Some(t) = tipo_ref {
            if reg.tipo != t {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"erro":"Só podes reordenar widgets do mesmo tipo."})),
                )
                    .into_response();
            }
        } else {
            tipo_ref = Some(reg.tipo);
        }
        let mut w = reg.to_widget(id);
        w.ordem = i as u16;
        w.actualizado_em = agora_unix();
        if let Err(e) = bd_alterar(bd.clone(), WidgetReg::from_widget(&w), id).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"erro": e.to_string()})),
            )
                .into_response();
        }
    }
    StatusCode::NO_CONTENT.into_response()
}

pub async fn remover_widget(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    if widget_do_dono(&state, id, sessao.labnetcol_id)
        .await
        .is_err()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"erro":"Widget não encontrado."})),
        )
            .into_response();
    }
    let Some(bd) = state.bd.get("widget") else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"erro":"Serviço indisponível"})),
        )
            .into_response();
    };
    match bd_remover(bd.clone(), id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"erro": e.to_string()})),
        )
            .into_response(),
    }
}
