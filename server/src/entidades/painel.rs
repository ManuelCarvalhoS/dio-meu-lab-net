use bytemuck::{Pod, Zeroable};
use meu_labnet_comum::PainelConfig;
use mcs_bd2::{CampoIndice, TipoCampo};

/// Layout mcs_bd2 — 24 bytes.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PainelConfigReg {
    pub utilizador: u64,
    pub tema: u8,
    pub layout: u8,
    pub _pad: [u8; 6],
    pub actualizado_em: u64,
}

const _: () = assert!(std::mem::size_of::<PainelConfigReg>() == 24);
const _: () = assert!(std::mem::size_of::<PainelConfigReg>() % 8 == 0);

pub const TAM_REG_PAINEL_U64: u64 = std::mem::size_of::<PainelConfigReg>() as u64;

pub const INDICES_PAINEL: &[CampoIndice] = &[CampoIndice {
    offset: 0,
    tamanho: 8,
    ficheiro: 1,
    tipo: TipoCampo::Inteiro,
}];

impl PainelConfigReg {
    pub fn from_config(c: &PainelConfig) -> Self {
        Self {
            utilizador: c.utilizador,
            tema: c.tema,
            layout: c.layout.clamp(1, 3),
            _pad: [0; 6],
            actualizado_em: c.actualizado_em,
        }
    }

    pub fn to_config(&self, n_reg: u64) -> PainelConfig {
        PainelConfig {
            n_reg,
            utilizador: self.utilizador,
            tema: self.tema,
            layout: self.layout.clamp(1, 3),
            actualizado_em: self.actualizado_em,
        }
    }
}
