use meu_labnet_comum::{
    LiveResumo, PainelConfig, PainelDto, PedidoPainelConfig, PedidoReordenar, PedidoWidget,
    SessaoMeuLabNet, Widget,
};
use reqwest::Client;
use serde::Deserialize;

pub struct ApiErro(pub String);

fn client() -> Client {
    Client::new()
}

pub fn api_url_padrao() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(origin) = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .filter(|s| !s.is_empty() && s != "null")
        {
            if origin.contains(":8092") {
                return "http://localhost:8093".into();
            }
            if origin.ends_with("labnetcol.pt") || origin.contains("labnet.") {
                return origin;
            }
        }
    }
    "http://localhost:8093".into()
}

async fn tratar(resp: reqwest::Response) -> String {
    let status = resp.status().as_u16();
    resp.json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| v.get("erro").and_then(|e| e.as_str()).map(String::from))
        .unwrap_or_else(|| format!("Erro {status}"))
}

pub async fn sso_labnetcol(api_url: &str, token_jwt: &str) -> Result<SessaoMeuLabNet, ApiErro> {
    let resp = client()
        .post(format!("{}/api/sso/labnetcol", api_url.trim_end_matches('/')))
        .json(&serde_json::json!({ "token": token_jwt }))
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}

pub async fn obter_painel(api_url: &str, token: &str) -> Result<PainelDto, ApiErro> {
    let resp = client()
        .get(format!("{}/api/painel", api_url.trim_end_matches('/')))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}

pub async fn actualizar_config(
    api_url: &str,
    token: &str,
    pedido: &PedidoPainelConfig,
) -> Result<PainelConfig, ApiErro> {
    let resp = client()
        .put(format!("{}/api/painel/config", api_url.trim_end_matches('/')))
        .bearer_auth(token)
        .json(pedido)
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}

pub async fn criar_widget(
    api_url: &str,
    token: &str,
    pedido: &PedidoWidget,
) -> Result<Widget, ApiErro> {
    let resp = client()
        .post(format!("{}/api/widgets", api_url.trim_end_matches('/')))
        .bearer_auth(token)
        .json(pedido)
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}

pub async fn remover_widget(api_url: &str, token: &str, id: u64) -> Result<(), ApiErro> {
    let resp = client()
        .delete(format!(
            "{}/api/widgets/{}",
            api_url.trim_end_matches('/'),
            id
        ))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if resp.status().is_success() || resp.status().as_u16() == 204 {
        Ok(())
    } else {
        Err(ApiErro(tratar(resp).await))
    }
}

pub async fn reordenar_widgets(
    api_url: &str,
    token: &str,
    ids: &[u64],
) -> Result<(), ApiErro> {
    let resp = client()
        .put(format!(
            "{}/api/widgets/reordenar",
            api_url.trim_end_matches('/')
        ))
        .bearer_auth(token)
        .json(&PedidoReordenar {
            ids: ids.to_vec(),
        })
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if resp.status().is_success() || resp.status().as_u16() == 204 {
        Ok(())
    } else {
        Err(ApiErro(tratar(resp).await))
    }
}

pub async fn live_resumo(api_url: &str, token: &str, slug: &str) -> Result<LiveResumo, ApiErro> {
    let resp = client()
        .get(format!(
            "{}/api/live/{}",
            api_url.trim_end_matches('/'),
            slug
        ))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct ModoAuth {
    pub dev_login: bool,
    pub labnetcol_url: String,
}

pub async fn auth_modo(api_url: &str) -> Result<ModoAuth, ApiErro> {
    let resp = client()
        .get(format!("{}/api/auth/modo", api_url.trim_end_matches('/')))
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}

pub async fn login_dev(api_url: &str, nome: &str) -> Result<SessaoMeuLabNet, ApiErro> {
    let resp = client()
        .post(format!("{}/api/auth/dev", api_url.trim_end_matches('/')))
        .json(&serde_json::json!({ "nome": nome }))
        .send()
        .await
        .map_err(|e| ApiErro(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ApiErro(tratar(resp).await));
    }
    resp.json().await.map_err(|e| ApiErro(e.to_string()))
}
