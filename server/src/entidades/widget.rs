use bytemuck::{Pod, Zeroable};
use meu_labnet_comum::{TipoWidget, Widget, TAM_CONTEUDO, TAM_TITULO};
use mcs_bd2::{CampoIndice, TipoCampo};

use crate::entidades::util::{arr_para_str, str_para_arr};

/// Layout mcs_bd2 — 352 bytes.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WidgetReg {
    pub utilizador: u64,
    pub tipo: u8,
    pub activo: u8,
    pub ordem: u16,
    pub _pad: [u8; 4],
    pub titulo: [u8; TAM_TITULO],
    pub conteudo: [u8; TAM_CONTEUDO],
    pub criado_em: u64,
    pub actualizado_em: u64,
}

const _: () = assert!(std::mem::size_of::<WidgetReg>() == 352);
const _: () = assert!(std::mem::size_of::<WidgetReg>() % 8 == 0);

pub const TAM_REG_WIDGET_U64: u64 = std::mem::size_of::<WidgetReg>() as u64;

pub const INDICES_WIDGET: &[CampoIndice] = &[CampoIndice {
    offset: 0,
    tamanho: 8,
    ficheiro: 1,
    tipo: TipoCampo::Inteiro,
}];

impl WidgetReg {
    pub fn from_widget(w: &Widget) -> Self {
        Self {
            utilizador: w.utilizador,
            tipo: w.tipo as u8,
            activo: u8::from(w.activo),
            ordem: w.ordem,
            _pad: [0; 4],
            titulo: str_para_arr(&w.titulo),
            conteudo: str_para_arr(&w.conteudo),
            criado_em: w.criado_em,
            actualizado_em: w.actualizado_em,
        }
    }

    pub fn to_widget(&self, n_reg: u64) -> Widget {
        Widget {
            n_reg,
            utilizador: self.utilizador,
            tipo: TipoWidget::from_u8(self.tipo),
            activo: self.activo != 0,
            ordem: self.ordem,
            titulo: arr_para_str(&self.titulo),
            conteudo: arr_para_str(&self.conteudo),
            criado_em: self.criado_em,
            actualizado_em: self.actualizado_em,
        }
    }
}
