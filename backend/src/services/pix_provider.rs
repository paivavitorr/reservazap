use base64::Engine;
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
    #[error("falha ao cancelar cobrança Pix: {0}")]
    CancelFailed(String),
}

#[async_trait::async_trait]
pub trait PixProvider: Send + Sync {
    async fn criar_cobranca(
        &self,
        valor_centavos: i64,
        descricao: &str,
        expiracao_segundos: u32,
    ) -> Result<PixCharge, PixError>;

    /// Cancela uma cobrança pendente. Usado quando o agendamento falha depois
    /// da cobrança já criada (evita cobrança órfã).
    async fn cancelar_cobranca(&self, txid: &str) -> Result<(), PixError>;
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
        let txid: String = Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(26)
            .collect();

        let copia_e_cola = br_code_pix(&txid, valor_centavos, descricao);
        let qr_code_base64 = gerar_qr_data_url(&copia_e_cola)?;

        Ok(PixCharge {
            txid,
            copia_e_cola,
            qr_code_base64: Some(qr_code_base64),
            valor_centavos,
            expiracao_segundos,
        })
    }

    async fn cancelar_cobranca(&self, _txid: &str) -> Result<(), PixError> {
        Ok(())
    }
}

fn gerar_qr_data_url(payload: &str) -> Result<String, PixError> {
    let code = qrcode::QrCode::new(payload.as_bytes())
        .map_err(|e| PixError::ChargeFailed(format!("qr inválido: {e}")))?;
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(280, 280)
        .build();
    let encoded = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{encoded}"))
}

/// Monta um BR Code (EMV® QRCPS-MPM) válido do Pix, com CRC16-CCITT.
fn br_code_pix(txid: &str, valor_centavos: i64, descricao: &str) -> String {
    fn tlv(id: &str, value: &str) -> String {
        format!("{id}{:02}{value}", value.len())
    }

    let valor = format!("{:.2}", valor_centavos as f64 / 100.0);
    let descricao_limpa: String = descricao
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '.' | ','))
        .take(40)
        .collect();
    let nome = "RESERVAZAP";
    let cidade = "SAO PAULO";

    let merchant_account = format!(
        "{}{}",
        tlv("00", "br.gov.bcb.pix"),
        tlv("01", "reservazap@mock.pix")
    );

    let mut payload = String::new();
    payload.push_str(&tlv("00", "01"));
    payload.push_str(&tlv("26", &merchant_account));
    payload.push_str(&tlv("52", "0000"));
    payload.push_str(&tlv("53", "986"));
    payload.push_str(&tlv("54", &valor));
    payload.push_str(&tlv("58", "BR"));
    payload.push_str(&tlv("59", nome));
    payload.push_str(&tlv("60", cidade));
    let additional = if descricao_limpa.is_empty() {
        tlv("05", txid)
    } else {
        format!("{}{}", tlv("05", txid), tlv("08", &descricao_limpa))
    };
    payload.push_str(&tlv("62", &additional));
    payload.push_str("6304");
    let crc = crc16_ccitt(payload.as_bytes());
    payload.push_str(&format!("{crc:04X}"));
    payload
}

fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn br_code_tem_crc16_valido() {
        let payload = br_code_pix("TXIDEXEMPLO12345678901234", 4500, "Sinal - Corte");
        assert!(payload.starts_with("00020126"));
        assert!(payload.contains("br.gov.bcb.pix"));
        assert!(payload.contains("540545.00"));
        assert!(payload[payload.len() - 8..].starts_with("6304"));

        let body = &payload[..payload.len() - 4];
        let crc_informado = u16::from_str_radix(&payload[payload.len() - 4..], 16).unwrap();
        assert_eq!(crc16_ccitt(body.as_bytes()), crc_informado);
    }

    #[test]
    fn qr_gera_data_url_svg() {
        let data_url = gerar_qr_data_url("00020126test").unwrap();
        assert!(data_url.starts_with("data:image/svg+xml;base64,"));
    }
}
