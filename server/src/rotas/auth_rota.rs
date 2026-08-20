use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use meu_labnet_comum::SessaoMeuLabNet;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::estado::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct SsoClaims {
    sub: u64,
    #[serde(default)]
    pseudonimo: String,
    exp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MeuLabNetClaims {
    sub: u64,
    nome: String,
    exp: u64,
}

#[derive(Deserialize)]
pub struct PedidoSso {
    pub token: String,
}

pub struct InfoSessao {
    pub labnetcol_id: u64,
    pub nome: String,
}

pub fn verificar_token(headers: &HeaderMap, secret: &str) -> Option<InfoSessao> {
    let auth = headers.get("Authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    let data = decode::<MeuLabNetClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?;
    Some(InfoSessao {
        labnetcol_id: data.claims.sub,
        nome: data.claims.nome,
    })
}

fn emitir_sessao_dados(
    state: &AppState,
    sub: u64,
    nome: String,
) -> Result<SessaoMeuLabNet, String> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 7 * 24 * 3600;
    let claims = MeuLabNetClaims {
        sub,
        nome: nome.clone(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| "Erro ao criar sessão.".to_string())?;
    Ok(SessaoMeuLabNet {
        token,
        labnetcol_id: sub,
        nome,
    })
}

pub fn sessao_de_sso_labnetcol(state: &AppState, token: &str) -> Result<SessaoMeuLabNet, String> {
    let data = decode::<SsoClaims>(
        token.trim(),
        &DecodingKey::from_secret(state.sso_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        log::warn!("SSO LabNetCol rejeitado: {e}");
        "Token SSO inválido ou expirado.".to_string()
    })?;
    let n_reg = data.claims.sub;
    let nome = {
        let p = data.claims.pseudonimo.trim();
        if p.is_empty() {
            format!("Utilizador {n_reg}")
        } else {
            p.to_string()
        }
    };
    emitir_sessao_dados(state, n_reg, nome)
}

pub async fn modo(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "dev_login": state.dev_login,
        "labnetcol_url": state.labnetcol_url,
    }))
}

#[derive(Deserialize)]
pub struct PedidoDev {
    #[serde(default)]
    pub nome: String,
    #[serde(default)]
    pub id: u64,
}

pub async fn login_dev(
    State(state): State<AppState>,
    Json(pedido): Json<PedidoDev>,
) -> impl IntoResponse {
    if !state.dev_login {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"erro": "Entrada directa desligada. Usa o LabNetCol."})),
        )
            .into_response();
    }
    let nome = {
        let n = pedido.nome.trim();
        if n.is_empty() {
            "Convidado".to_string()
        } else {
            n.to_string()
        }
    };
    let id = if pedido.id != 0 { pedido.id } else { 1 };
    match emitir_sessao_dados(&state, id, nome) {
        Ok(s) => (StatusCode::OK, Json(s)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"erro": e})),
        )
            .into_response(),
    }
}

pub async fn sso_labnetcol(
    State(state): State<AppState>,
    Json(pedido): Json<PedidoSso>,
) -> impl IntoResponse {
    match sessao_de_sso_labnetcol(&state, &pedido.token) {
        Ok(s) => (StatusCode::OK, Json(s)).into_response(),
        Err(msg) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"erro": msg})),
        )
            .into_response(),
    }
}

pub async fn eu(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    match verificar_token(&headers, &state.jwt_secret) {
        Some(s) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "labnetcol_id": s.labnetcol_id,
                "nome": s.nome,
            })),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"erro":"Não autenticado"})),
        )
            .into_response(),
    }
}
