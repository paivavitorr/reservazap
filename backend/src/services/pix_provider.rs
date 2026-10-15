use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixCharge {
    pub txid: String,
    pub copia_e_cola: String,
    pub qr_code_base64: Option<String>,
    pub valor_centavos: i64,
    pub expiracao_segundos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixStatus {
    Pending,
    Paid,
    Cancelled,
    Expired,
    Unknown,
}

#[derive(Debug, thiserror::Error)]
pub enum PixError {
    #[error("falha ao criar cobrança Pix: {0}")]
    ChargeFailed(String),
    #[error("falha ao cancelar cobrança Pix: {0}")]
    CancelFailed(String),
    #[error("falha ao consultar cobrança Pix: {0}")]
    ConsultFailed(String),
    #[error("configuração do provedor Pix inválida: {0}")]
    Config(String),
}

#[async_trait::async_trait]
pub trait PixProvider: Send + Sync {
    /// Nome do provedor (registrado em `payments.provider`).
    fn nome(&self) -> &'static str;

    async fn criar_cobranca(
        &self,
        valor_centavos: i64,
        descricao: &str,
        expiracao_segundos: u32,
        payer_email: Option<&str>,
    ) -> Result<PixCharge, PixError>;

    /// Cancela uma cobrança pendente. Usado quando o agendamento falha depois
    /// da cobrança já criada (evita cobrança órfã).
    async fn cancelar_cobranca(&self, txid: &str) -> Result<(), PixError>;

    /// Situação atual da cobrança — usada para conferir webhooks e reconciliar.
    async fn consultar_cobranca(&self, txid: &str) -> Result<PixStatus, PixError>;
}

/// Monta o provider configurado por ambiente (`PIX_PROVIDER`).
pub fn from_env() -> Arc<dyn PixProvider> {
    match std::env::var("PIX_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .as_str()
    {
        "mercadopago" => {
            let token = std::env::var("MERCADOPAGO_ACCESS_TOKEN").unwrap_or_default();
            if token.is_empty() {
                // nunca cair silenciosamente no mock em produção: o provider sem
                // token falha as cobranças com erro claro em vez de QR falso
                tracing::error!(
                    "PIX_PROVIDER=mercadopago sem MERCADOPAGO_ACCESS_TOKEN — cobranças falharão"
                );
            }
            Arc::new(MercadoPagoPixProvider::new(token))
        }
        _ => Arc::new(MockPixProvider),
    }
}

// ───────────────────────────── Mock (dev/demo) ─────────────────────────────

pub struct MockPixProvider;

#[async_trait::async_trait]
impl PixProvider for MockPixProvider {
    fn nome(&self) -> &'static str {
        "mock"
    }

    async fn criar_cobranca(
        &self,
        valor_centavos: i64,
        descricao: &str,
        expiracao_segundos: u32,
        _payer_email: Option<&str>,
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

    async fn consultar_cobranca(&self, _txid: &str) -> Result<PixStatus, PixError> {
        Ok(PixStatus::Pending)
    }
}

// ──────────────────────────── Mercado Pago ────────────────────────────

pub struct MercadoPagoPixProvider {
    access_token: String,
    client: reqwest::Client,
}

fn http_client() -> reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new).clone()
}

impl MercadoPagoPixProvider {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            client: http_client(),
        }
    }

    fn url(&self, caminho: &str) -> String {
        format!("https://api.mercadopago.com{caminho}")
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.access_token)
    }
}

#[derive(Debug, Deserialize)]
struct MpPaymentResponse {
    id: serde_json::Value,
    status: Option<String>,
    #[serde(default)]
    point_of_interaction: Option<MpPointOfInteraction>,
}

#[derive(Debug, Deserialize)]
struct MpPointOfInteraction {
    #[serde(default)]
    transaction_data: Option<MpTransactionData>,
}

#[derive(Debug, Deserialize)]
struct MpTransactionData {
    qr_code: Option<String>,
    qr_code_base64: Option<String>,
}

fn mapear_status(status: Option<&str>) -> PixStatus {
    match status.unwrap_or_default() {
        "approved" => PixStatus::Paid,
        "cancelled" | "refunded" | "charged_back" => PixStatus::Cancelled,
        "expired" | "rejected" => PixStatus::Expired,
        "pending" | "in_process" | "in_mediation" | "authorized" => PixStatus::Pending,
        _ => PixStatus::Unknown,
    }
}

#[async_trait::async_trait]
impl PixProvider for MercadoPagoPixProvider {
    fn nome(&self) -> &'static str {
        "mercadopago"
    }

    async fn criar_cobranca(
        &self,
        valor_centavos: i64,
        descricao: &str,
        expiracao_segundos: u32,
        payer_email: Option<&str>,
    ) -> Result<PixCharge, PixError> {
        if self.access_token.is_empty() {
            return Err(PixError::Config(
                "MERCADOPAGO_ACCESS_TOKEN ausente".to_string(),
            ));
        }

        let valor = valor_centavos as f64 / 100.0;
        // a API do Mercado Pago só aceita expiração entre 30min e 30 dias;
        // o hold do agendamento continua sendo o que libera o horário
        let seg = expiracao_segundos.clamp(30 * 60, 30 * 24 * 60 * 60);
        let expira_em = chrono::Utc::now() + chrono::Duration::seconds(i64::from(seg));
        let email = payer_email
            .filter(|e| e.contains('@'))
            .unwrap_or("cliente@reservazap.com.br");
        let body = serde_json::json!({
            "transaction_amount": valor,
            "description": descricao,
            "payment_method_id": "pix",
            "date_of_expiration": expira_em.to_rfc3339(),
            "payer": { "email": email },
        });

        let resp = self
            .client
            .post(self.url("/v1/payments"))
            .header("Authorization", self.auth())
            .json(&body)
            .send()
            .await
            .map_err(|e| PixError::ChargeFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let texto = resp.text().await.unwrap_or_default();
            return Err(PixError::ChargeFailed(format!("HTTP: {texto}")));
        }

        let pagamento: MpPaymentResponse = resp
            .json()
            .await
            .map_err(|e| PixError::ChargeFailed(e.to_string()))?;

        let txid = pagamento.id.to_string().trim_matches('"').to_string();
        let dados = pagamento
            .point_of_interaction
            .and_then(|p| p.transaction_data)
            .unwrap_or(MpTransactionData {
                qr_code: None,
                qr_code_base64: None,
            });

        let copia_e_cola = dados
            .qr_code
            .ok_or_else(|| PixError::ChargeFailed("resposta sem qr_code".to_string()))?;

        Ok(PixCharge {
            txid,
            copia_e_cola,
            qr_code_base64: dados.qr_code_base64.map(|q| {
                if q.starts_with("data:") {
                    q
                } else {
                    format!("data:image/png;base64,{q}")
                }
            }),
            valor_centavos,
            expiracao_segundos,
        })
    }

    async fn cancelar_cobranca(&self, txid: &str) -> Result<(), PixError> {
        let resp = self
            .client
            .put(self.url(&format!("/v1/payments/{txid}")))
            .header("Authorization", self.auth())
            .json(&serde_json::json!({ "status": "cancelled" }))
            .send()
            .await
            .map_err(|e| PixError::CancelFailed(e.to_string()))?;
        if !resp.status().is_success() {
            let texto = resp.text().await.unwrap_or_default();
            return Err(PixError::CancelFailed(format!("HTTP: {texto}")));
        }
        Ok(())
    }

    async fn consultar_cobranca(&self, txid: &str) -> Result<PixStatus, PixError> {
        let resp = self
            .client
            .get(self.url(&format!("/v1/payments/{txid}")))
            .header("Authorization", self.auth())
            .send()
            .await
            .map_err(|e| PixError::ConsultFailed(e.to_string()))?;
        if !resp.status().is_success() {
            let texto = resp.text().await.unwrap_or_default();
            return Err(PixError::ConsultFailed(format!("HTTP: {texto}")));
        }
        let pagamento: MpPaymentResponse = resp
            .json()
            .await
            .map_err(|e| PixError::ConsultFailed(e.to_string()))?;
        Ok(mapear_status(pagamento.status.as_deref()))
    }
}

/// Valida a assinatura `x-signature` do webhook do Mercado Pago.
///
/// `manifest = "id:{data_id};request-id:{request_id};ts:{timestamp};"` assinado
/// com HMAC-SHA256 (hex) usando o webhook secret.
pub fn assinatura_webhook_valida(
    secret: &str,
    data_id: &str,
    request_id: &str,
    timestamp: &str,
    v1_hex: &str,
) -> bool {
    if secret.is_empty() || v1_hex.is_empty() {
        return false;
    }
    let manifest = format!("id:{data_id};request-id:{request_id};ts:{timestamp};");
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(manifest.as_bytes());
    let informado = match hex::decode(v1_hex.trim().to_lowercase()) {
        Ok(b) => b,
        Err(_) => return false,
    };
    // comparação em tempo constante
    mac.verify_slice(&informado).is_ok()
}

/// Extrai `ts` e `v1` do header `x-signature` (`ts=...,v1=...`).
pub fn parse_x_signature(header: &str) -> Option<(String, String)> {
    let mut ts = None;
    let mut v1 = None;
    for parte in header.split(',') {
        let parte = parte.trim();
        if let Some(v) = parte.strip_prefix("ts=") {
            ts = Some(v.to_string());
        } else if let Some(v) = parte.strip_prefix("v1=") {
            v1 = Some(v.to_string());
        }
    }
    Some((ts?, v1?))
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

    #[test]
    fn assinatura_webhook_e_validada() {
        // vetor calculado com o mesmo algoritmo
        let secret = "segredo-teste";
        let data_id = "123456";
        let request_id = "req-abc";
        let ts = "1700000000";
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("id:{data_id};request-id:{request_id};ts:{ts};").as_bytes());
        let v1 = hex::encode(mac.finalize().into_bytes());

        assert!(assinatura_webhook_valida(
            secret, data_id, request_id, ts, &v1
        ));
        assert!(!assinatura_webhook_valida(
            secret,
            data_id,
            request_id,
            "9999999999",
            &v1
        ));
        assert!(!assinatura_webhook_valida(
            secret, "outro", request_id, ts, &v1
        ));
        assert!(!assinatura_webhook_valida(
            "outro-secret",
            data_id,
            request_id,
            ts,
            &v1
        ));
        assert!(!assinatura_webhook_valida(
            secret,
            data_id,
            request_id,
            ts,
            "zz-not-hex"
        ));
        assert!(!assinatura_webhook_valida("", data_id, request_id, ts, &v1));
    }

    #[test]
    fn parse_do_header_x_signature() {
        let (ts, v1) = parse_x_signature("ts=1700000000,v1=abcdef").unwrap();
        assert_eq!(ts, "1700000000");
        assert_eq!(v1, "abcdef");
        assert!(parse_x_signature("ts=123").is_none());
        assert!(parse_x_signature("v1=abc").is_none());
    }

    #[test]
    fn mapeia_status_do_mercado_pago() {
        assert_eq!(mapear_status(Some("approved")), PixStatus::Paid);
        assert_eq!(mapear_status(Some("pending")), PixStatus::Pending);
        assert_eq!(mapear_status(Some("cancelled")), PixStatus::Cancelled);
        assert_eq!(mapear_status(Some("expired")), PixStatus::Expired);
        assert_eq!(mapear_status(None), PixStatus::Unknown);
    }
}
