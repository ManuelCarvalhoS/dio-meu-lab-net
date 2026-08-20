use axum::{http::Method, Router};
use crate::entidades::{INDICES_PAINEL, INDICES_WIDGET, TAM_REG_PAINEL_U64, TAM_REG_WIDGET_U64};
use mcs_bd2::{abrir_ficheiros, CampoIndice};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod entidades;
mod estado;
mod rotas;

fn entities() -> Vec<(String, u32, Vec<String>, u64, &'static [CampoIndice])> {
    vec![
        (
            "painel".into(),
            1,
            vec![
                "painel.dad".into(),
                "painel.h1".into(),
                "painel.h2".into(),
                "painel.i1".into(),
            ],
            TAM_REG_PAINEL_U64,
            INDICES_PAINEL,
        ),
        (
            "widget".into(),
            2,
            vec![
                "widget.dad".into(),
                "widget.h1".into(),
                "widget.h2".into(),
                "widget.i1".into(),
            ],
            TAM_REG_WIDGET_U64,
            INDICES_WIDGET,
        ),
    ]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    simple_logger::init_with_level(log::Level::Info).ok();

    let porta: u16 = std::env::var("MEU_LABNET_PORT")
        .unwrap_or_else(|_| "8093".into())
        .parse()
        .unwrap_or(8093);

    let data_dir =
        std::env::var("MEU_LABNET_DATA_DIR").unwrap_or_else(|_| "./data".into());
    std::fs::create_dir_all(&data_dir)?;

    let bd = abrir_ficheiros(&data_dir, &data_dir, entities()).expect("Falha ao abrir BD Meu LabNet");
    log::info!("BD Meu LabNet aberta em {data_dir}");

    let jwt_secret = std::env::var("MEU_LABNET_JWT_SECRET")
        .unwrap_or_else(|_| "meu-labnet-jwt-dev".into());
    let dev_login = match std::env::var("MEU_LABNET_DEV_LOGIN") {
        Ok(v) => !matches!(v.to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"),
        Err(_) => true,
    };

    let state = estado::AppState {
        bd: Arc::new(bd),
        sso_secret: std::env::var("LABNETCOL_SECRET")
            .unwrap_or_else(|_| "labnetcol-sso-dev-secret".into()),
        jwt_secret,
        labnetcol_url: std::env::var("LABNETCOL_FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:8080".into()),
        dev_login,
        lista_url: std::env::var("LISTA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8088".into()),
        lista_jwt_secret: std::env::var("LISTA_JWT_SECRET")
            .unwrap_or_else(|_| "lista-jwt-dev".into()),
        agenda_url: std::env::var("AGENDA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8091".into()),
        agenda_jwt_secret: std::env::var("AGENDA_JWT_SECRET")
            .unwrap_or_else(|_| "agenda-jwt-dev".into()),
        encripta_url: std::env::var("ENCRIPTA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8090".into()),
    };

    if state.dev_login {
        log::warn!("Entrada directa activa (MEU_LABNET_DEV_LOGIN)");
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", rotas::api_router(state))
        .layer(cors);

    let addr = format!("127.0.0.1:{porta}");
    log::info!("Meu LabNet API a escutar em http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
