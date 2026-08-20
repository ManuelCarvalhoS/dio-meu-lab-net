use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessaoMeuLabNet {
    pub token: String,
    pub labnetcol_id: u64,
    pub nome: String,
}
