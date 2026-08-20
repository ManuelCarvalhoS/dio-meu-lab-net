//! Resumo live das apps LabNet (BFF).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiveResumo {
    pub slug: String,
    pub ok: bool,
    /// Linha curta para o cartão (ex.: "3/12 · ~12,50 €").
    pub linha: String,
    #[serde(default)]
    pub detalhe: Option<String>,
}

impl LiveResumo {
    pub fn ok(slug: &str, linha: impl Into<String>) -> Self {
        Self {
            slug: slug.to_string(),
            ok: true,
            linha: linha.into(),
            detalhe: None,
        }
    }

    pub fn aviso(slug: &str, linha: impl Into<String>) -> Self {
        Self {
            slug: slug.to_string(),
            ok: false,
            linha: linha.into(),
            detalhe: None,
        }
    }
}
