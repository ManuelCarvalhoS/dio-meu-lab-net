use serde::{Deserialize, Serialize};

use crate::util::agora_unix;

pub const TAM_CONTEUDO: usize = 256;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TipoWidget {
    #[default]
    Link = 1,
    Nota = 2,
    Atalho = 3,
}

impl TipoWidget {
    pub fn from_u8(v: u8) -> Self {
        match v {
            2 => Self::Nota,
            3 => Self::Atalho,
            _ => Self::Link,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    pub n_reg: u64,
    pub utilizador: u64,
    pub tipo: TipoWidget,
    pub activo: bool,
    pub ordem: u16,
    pub titulo: String,
    pub conteudo: String,
    pub criado_em: u64,
    pub actualizado_em: u64,
}

impl Widget {
    pub fn novo(utilizador: u64, tipo: TipoWidget, titulo: &str, conteudo: &str, ordem: u16) -> Self {
        let agora = agora_unix();
        Self {
            n_reg: 0,
            utilizador,
            tipo,
            activo: true,
            ordem,
            titulo: titulo.trim().to_string(),
            conteudo: conteudo.trim().to_string(),
            criado_em: agora,
            actualizado_em: agora,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PedidoWidget {
    pub tipo: TipoWidget,
    pub titulo: String,
    pub conteudo: String,
    #[serde(default)]
    pub ordem: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PedidoWidgetPatch {
    pub titulo: Option<String>,
    pub conteudo: Option<String>,
    pub ordem: Option<u16>,
    pub activo: Option<bool>,
}

/// Nova ordem dos widgets (ids do mesmo tipo, do topo para baixo).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PedidoReordenar {
    pub ids: Vec<u64>,
}
