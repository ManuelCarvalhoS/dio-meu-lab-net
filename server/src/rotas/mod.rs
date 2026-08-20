use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::estado::AppState;

pub mod auth_rota;
pub mod live_rota;
pub mod painel_rota;
pub mod widget_rota;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/sso/labnetcol", post(auth_rota::sso_labnetcol))
        .route("/auth/modo", get(auth_rota::modo))
        .route("/auth/dev", post(auth_rota::login_dev))
        .route("/eu", get(auth_rota::eu))
        .route("/painel", get(painel_rota::obter_painel))
        .route("/painel/config", put(painel_rota::actualizar_config))
        .route("/live/{slug}", get(live_rota::obter_live))
        .route("/widgets", post(widget_rota::criar_widget))
        .route("/widgets/reordenar", put(widget_rota::reordenar_widgets))
        .route("/widgets/{id}", put(widget_rota::editar_widget))
        .route("/widgets/{id}", delete(widget_rota::remover_widget))
        .with_state(state)
}
