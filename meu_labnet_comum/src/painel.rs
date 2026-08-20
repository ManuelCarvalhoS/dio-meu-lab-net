use serde::{Deserialize, Serialize};

use crate::util::agora_unix;

pub const TAM_TITULO: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PainelConfig {
    pub n_reg: u64,
    pub utilizador: u64,
    pub tema: u8,
    pub layout: u8,
    pub actualizado_em: u64,
}

impl PainelConfig {
    pub fn padrao(utilizador: u64) -> Self {
        Self {
            n_reg: 0,
            utilizador,
            tema: 0,
            layout: 3,
            actualizado_em: agora_unix(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PedidoPainelConfig {
    pub tema: Option<u8>,
    pub layout: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PainelDto {
    pub config: PainelConfig,
    pub widgets: Vec<crate::widget::Widget>,
}
