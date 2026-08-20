use mcs_bd2::estrutura::EntityFiles;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type BdMapa = HashMap<String, Arc<Mutex<EntityFiles>>>;

#[derive(Clone)]
pub struct AppState {
    pub bd: Arc<BdMapa>,
    pub sso_secret: String,
    pub jwt_secret: String,
    pub labnetcol_url: String,
    pub dev_login: bool,
    pub lista_url: String,
    pub lista_jwt_secret: String,
    pub agenda_url: String,
    pub agenda_jwt_secret: String,
    pub encripta_url: String,
}
