use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixCharge {
    pub txid: String,
    pub copia_e_cola: String,
    pub qr_code_base64: Option<String>,
    pub valor_centavos: i64,
    pub expiracao_segundos: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum PixError {
    #[error("falha ao criar cobrança Pix: {0}")]
    ChargeFailed(String),
}

#[async_trait::async_trait]
pub trait PixProvider: Send + Sync {
    async fn criar_cobranca(
        &self,
        valor_centavos: i64,
        descricao: &str,
        expiracao_segundos: u32,
    ) -> Result<PixCharge, PixError>;
}

pub struct MockPixProvider;

#[async_trait::async_trait]
impl PixProvider for MockPixProvider {
    async fn criar_cobranca(
        &self,
        valor_centavos: i64,
        descricao: &str,
        expiracao_segundos: u32,
    ) -> Result<PixCharge, PixError> {
        let txid = Uuid::new_v4().to_string().replace('-', "")[..26].to_string();
        Ok(PixCharge {
            txid,
            copia_e_cola: format!("00020126580014br.gov.bcb.pix0136{}520400005303986540{:.2}5802BR5925RESERVAZAP6009SAO PAULO62070503***", Uuid::new_v4(), valor_centavos as f64 / 100.0),
            qr_code_base64: Some("data:image/png;base64,iVBOR...".to_string()),
            valor_centavos,
            expiracao_segundos,
        })
    }
}