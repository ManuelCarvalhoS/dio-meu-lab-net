//! BFF: resumos live das apps satélite (Lista, Agenda, Encripta).

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use meu_labnet_comum::LiveResumo;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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

#[derive(Debug, Serialize)]
struct PeerClaims {
    sub: u64,
    nome: String,
    tipo: u8,
    admin: bool,
    exp: u64,
}

fn emitir_peer_jwt(secret: &str, sub: u64, nome: &str) -> Result<String, String> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 3600;
    let claims = PeerClaims {
        sub,
        nome: nome.to_string(),
        tipo: 0,
        admin: false,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())
}

fn euros(cent: u32) -> String {
    format!("{},{:02} €", cent / 100, cent % 100)
}

#[derive(Deserialize)]
struct ListaResumo {
    n_reg: u64,
    nome: String,
    #[serde(default)]
    activa: bool,
}

#[derive(Deserialize)]
struct TotaisLista {
    n_produtos: usize,
    n_comprados: usize,
    #[serde(default)]
    estimado_lista_cent: u32,
}

async fn resumo_lista(state: &AppState, uid: u64, nome: &str) -> LiveResumo {
    let Ok(token) = emitir_peer_jwt(&state.lista_jwt_secret, uid, nome) else {
        return LiveResumo::aviso("lista", "Lista indisponível");
    };
    let base = state.lista_url.trim_end_matches('/');
    let client = reqwest::Client::new();
    let listas = match client
        .get(format!("{base}/api/listas"))
        .bearer_auth(&token)
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r.json::<Vec<ListaResumo>>().await.unwrap_or_default(),
        Ok(_) => return LiveResumo::aviso("lista", "Lista: sessão rejeitada"),
        Err(_) => return LiveResumo::aviso("lista", "Lista offline"),
    };
    if listas.is_empty() {
        return LiveResumo::ok("lista", "Sem listas ainda");
    }
    let lista = listas
        .iter()
        .find(|l| l.activa)
        .unwrap_or(&listas[0]);
    let totais = match client
        .get(format!("{base}/api/listas/{}/totais", lista.n_reg))
        .bearer_auth(&token)
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r.json::<TotaisLista>().await.ok(),
        _ => None,
    };
    let nome_curto = if lista.nome.chars().count() > 18 {
        format!("{}…", lista.nome.chars().take(16).collect::<String>())
    } else {
        lista.nome.clone()
    };
    match totais {
        Some(t) if t.n_produtos > 0 => {
            let mut linha = format!("{nome_curto} · {}/{}", t.n_comprados, t.n_produtos);
            if t.estimado_lista_cent > 0 {
                linha.push_str(&format!(" · ~{}", euros(t.estimado_lista_cent)));
            }
            LiveResumo::ok("lista", linha)
        }
        Some(_) => LiveResumo::ok("lista", format!("{nome_curto} · vazia")),
        None => LiveResumo::ok("lista", nome_curto),
    }
}

async fn resumo_agenda(state: &AppState, uid: u64, nome: &str) -> LiveResumo {
    let Ok(token) = emitir_peer_jwt(&state.agenda_jwt_secret, uid, nome) else {
        return LiveResumo::aviso("agenda", "Agenda indisponível");
    };
    let base = state.agenda_url.trim_end_matches('/');
    let (ano, mes) = chrono_naive_ym();
    let client = reqwest::Client::new();
    let url = format!("{base}/api/agenda/mes?ano={ano}&mes={mes}");
    match client
        .get(&url)
        .bearer_auth(&token)
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            let n = r
                .json::<Vec<serde_json::Value>>()
                .await
                .map(|v| v.len())
                .unwrap_or(0);
            let mes_nome = nome_mes(mes);
            if n == 0 {
                LiveResumo::ok("agenda", format!("Nada em {mes_nome}"))
            } else if n == 1 {
                LiveResumo::ok("agenda", format!("1 evento em {mes_nome}"))
            } else {
                LiveResumo::ok("agenda", format!("{n} eventos em {mes_nome}"))
            }
        }
        Ok(r) if r.status().as_u16() == 401 => {
            LiveResumo::aviso("agenda", "Abrir Agenda para activar")
        }
        Ok(_) => LiveResumo::aviso("agenda", "Agenda: erro"),
        Err(_) => LiveResumo::aviso("agenda", "Agenda offline"),
    }
}

fn chrono_naive_ym() -> (u16, u8) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = secs / 86_400;
    let (y, m, _d) = civil_from_days(days);
    (y as u16, m as u8)
}

/// Howard Hinnant (domínio público): dias desde 1970-01-01 → Y-M-D.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

fn nome_mes(m: u8) -> &'static str {
    match m {
        1 => "jan",
        2 => "fev",
        3 => "mar",
        4 => "abr",
        5 => "mai",
        6 => "jun",
        7 => "jul",
        8 => "ago",
        9 => "set",
        10 => "out",
        11 => "nov",
        12 => "dez",
        _ => "?",
    }
}

async fn resumo_encripta(state: &AppState) -> LiveResumo {
    let base = state.encripta_url.trim_end_matches('/');
    match reqwest::Client::new()
        .get(format!("{base}/snapshots"))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(r) if r.status().as_u16() == 401 || r.status().is_success() => {
            LiveResumo::ok("encripta", "Backup online · abrir")
        }
        Ok(_) => LiveResumo::aviso("encripta", "Encripta: erro"),
        Err(_) => LiveResumo::aviso("encripta", "Encripta offline"),
    }
}

/// GET /api/live/{slug}
pub async fn obter_live(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let sessao = exige!(headers, state);
    let slug = slug.trim().to_ascii_lowercase();
    let resumo = match slug.as_str() {
        "lista" => resumo_lista(&state, sessao.labnetcol_id, &sessao.nome).await,
        "agenda" | "superagenda" => {
            resumo_agenda(&state, sessao.labnetcol_id, &sessao.nome).await
        }
        "encripta" => resumo_encripta(&state).await,
        "labnetcol" => LiveResumo::ok("labnetcol", "Portal · identidade"),
        _ => LiveResumo::aviso(&slug, "Sem resumo live"),
    };
    Json(resumo).into_response()
}
