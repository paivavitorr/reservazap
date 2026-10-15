use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use std::sync::Arc;

use crate::models::_entities::appointments;
use crate::models::_entities::professionals;
use crate::models::_entities::services;
use crate::models::_entities::tenants;
use crate::models::appointments::AppointmentModel;

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("falha ao enviar WhatsApp: {0}")]
    EnvioFalhou(String),
    #[error("configuração do provedor de notificação inválida: {0}")]
    Config(String),
}

#[async_trait::async_trait]
pub trait NotificationProvider: Send + Sync {
    fn nome(&self) -> &'static str;

    /// Envia mensagem de texto para um WhatsApp (número no formato 55DDDNÚMERO).
    async fn enviar_whatsapp(&self, numero: &str, mensagem: &str) -> Result<(), NotificationError>;
}

/// Provider de desenvolvimento: apenas registra a mensagem no log.
pub struct MockNotificationProvider;

#[async_trait::async_trait]
impl NotificationProvider for MockNotificationProvider {
    fn nome(&self) -> &'static str {
        "mock"
    }

    async fn enviar_whatsapp(&self, numero: &str, mensagem: &str) -> Result<(), NotificationError> {
        tracing::info!(numero, mensagem, "mock whatsapp enviado");
        Ok(())
    }
}

/// Evolution API (auto-hospedada) — envio de texto via `/message/sendText`.
pub struct EvolutionApiProvider {
    base_url: String,
    api_key: String,
    instance: String,
    client: reqwest::Client,
}

impl EvolutionApiProvider {
    pub fn new(base_url: String, api_key: String, instance: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            instance,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait::async_trait]
impl NotificationProvider for EvolutionApiProvider {
    fn nome(&self) -> &'static str {
        "evolution"
    }

    async fn enviar_whatsapp(&self, numero: &str, mensagem: &str) -> Result<(), NotificationError> {
        let body = serde_json::json!({
            "number": numero,
            "text": mensagem,
        });
        let resp = self
            .client
            .post(format!(
                "{}/message/sendText/{}",
                self.base_url, self.instance
            ))
            .header("apikey", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| NotificationError::EnvioFalhou(e.to_string()))?;
        if !resp.status().is_success() {
            let texto = resp.text().await.unwrap_or_default();
            return Err(NotificationError::EnvioFalhou(format!("HTTP: {texto}")));
        }
        Ok(())
    }
}

/// Provider por ambiente (`NOTIFICATION_PROVIDER`): mock | evolution.
pub fn from_env() -> Arc<dyn NotificationProvider> {
    match std::env::var("NOTIFICATION_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .as_str()
    {
        "evolution" => {
            let base_url = std::env::var("EVOLUTION_API_URL").unwrap_or_default();
            let api_key = std::env::var("EVOLUTION_API_KEY").unwrap_or_default();
            let instance = std::env::var("EVOLUTION_INSTANCE").unwrap_or_default();
            if base_url.is_empty() || api_key.is_empty() || instance.is_empty() {
                tracing::error!(
                    "NOTIFICATION_PROVIDER=evolution sem EVOLUTION_API_URL/EVOLUTION_API_KEY/EVOLUTION_INSTANCE — envios falharão"
                );
            }
            Arc::new(EvolutionApiProvider::new(base_url, api_key, instance))
        }
        _ => Arc::new(MockNotificationProvider),
    }
}

/// Normaliza telefone para o padrão internacional do WhatsApp (55DDDNÚMERO).
pub fn normalizar_telefone(raw: &str) -> Option<String> {
    let digitos: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    match digitos.len() {
        10 | 11 => Some(format!("55{digitos}")),
        12 | 13 if digitos.starts_with("55") => Some(digitos),
        _ => None,
    }
}

// ─────────────────────────── Templates de mensagem ───────────────────────────

pub fn formatar_data_hora(timezone: &str, instante: DateTime<Utc>) -> String {
    let tz = Tz::from_str(timezone).unwrap_or(chrono_tz::UTC);
    instante
        .with_timezone(&tz)
        .format("%d/%m/%Y às %H:%M")
        .to_string()
}

pub fn mensagem_confirmacao(
    cliente: &str,
    estabelecimento: &str,
    servico: &str,
    profissional: &str,
    data_hora: &str,
) -> String {
    format!(
        "✅ *Agendamento confirmado!*\n\n\
         Olá, {cliente}! Seu horário na *{estabelecimento}* está garantido:\n\n\
         Serviço: {servico}\n\
         Profissional: {profissional}\n\
         Quando: {data_hora}\n\n\
         Chegue com alguns minutos de antecedência. Até logo!"
    )
}

pub fn mensagem_lembrete_24h(
    cliente: &str,
    estabelecimento: &str,
    servico: &str,
    data_hora: &str,
) -> String {
    format!(
        "📅 *Lembrete de agendamento*\n\n\
         Olá, {cliente}! Passando para lembrar do seu horário na *{estabelecimento}*:\n\n\
         Serviço: {servico}\n\
         Quando: {data_hora}\n\n\
         Precisa remarcar? Responda esta mensagem."
    )
}

pub fn mensagem_lembrete_1h(
    cliente: &str,
    estabelecimento: &str,
    servico: &str,
    data_hora: &str,
) -> String {
    format!(
        "⏰ *Seu horário é em 1 hora!*\n\n\
         Olá, {cliente}! Seu atendimento de *{servico}* na *{estabelecimento}* será em breve — {data_hora}.\n\n\
         Esperamos você!"
    )
}

pub fn mensagem_horario_liberado(cliente: &str, estabelecimento: &str) -> String {
    format!(
        "Olá, {cliente}! O sinal do seu agendamento na *{estabelecimento}* não foi pago dentro do \
         prazo e o horário foi liberado. Se ainda quiser atender, é só agendar novamente pelo link."
    )
}

// ─────────────────────────── Orquestração sobre o banco ───────────────────────────

/// Envia a confirmação para o cliente de um agendamento confirmado.
pub async fn notificar_confirmacao(
    db: &DatabaseConnection,
    appt: &appointments::Model,
    provider: &Arc<dyn NotificationProvider>,
) {
    if let Err(e) = enviar_para_agendamento(db, appt, provider, |dados| {
        mensagem_confirmacao(
            &appt.client_name,
            &dados.estabelecimento,
            &dados.servico,
            &dados.profissional,
            &dados.data_hora,
        )
    })
    .await
    {
        tracing::warn!(appointment_id = %appt.id, error = %e, "falha ao notificar confirmação");
    }
}

/// Envia o aviso de horário liberado (hold expirado).
pub async fn notificar_horario_liberado(
    db: &DatabaseConnection,
    appt: &appointments::Model,
    provider: &Arc<dyn NotificationProvider>,
) {
    if let Err(e) = enviar_para_agendamento(db, appt, provider, |dados| {
        mensagem_horario_liberado(&appt.client_name, &dados.estabelecimento)
    })
    .await
    {
        tracing::warn!(appointment_id = %appt.id, error = %e, "falha ao notificar expiração");
    }
}

struct DadosAgendamento {
    estabelecimento: String,
    servico: String,
    profissional: String,
    data_hora: String,
}

async fn enviar_para_agendamento(
    db: &DatabaseConnection,
    appt: &appointments::Model,
    provider: &Arc<dyn NotificationProvider>,
    montar: impl Fn(&DadosAgendamento) -> String,
) -> Result<(), NotificationError> {
    let numero = normalizar_telefone(&appt.client_phone).ok_or_else(|| {
        NotificationError::Config(format!("telefone inválido: {}", appt.client_phone))
    })?;

    use sea_orm::EntityTrait;

    let tenant = tenants::Entity::find_by_id(appt.tenant_id)
        .one(db)
        .await
        .map_err(|e| NotificationError::Config(e.to_string()))?
        .ok_or_else(|| NotificationError::Config("tenant não encontrado".to_string()))?;

    let servico = services::Entity::find_by_id(appt.service_id)
        .one(db)
        .await
        .map_err(|e| NotificationError::Config(e.to_string()))?
        .map(|s| s.name)
        .unwrap_or_else(|| "Serviço".to_string());

    let profissional = professionals::Entity::find_by_id(appt.professional_id)
        .one(db)
        .await
        .map_err(|e| NotificationError::Config(e.to_string()))?
        .map(|p| p.name)
        .unwrap_or_else(|| "Profissional".to_string());

    let dados = DadosAgendamento {
        estabelecimento: tenant.name.clone(),
        servico,
        profissional,
        data_hora: formatar_data_hora(&tenant.timezone, appt.starts_at.with_timezone(&Utc)),
    };

    provider.enviar_whatsapp(&numero, &montar(&dados)).await
}

/// Lembretes de 24h e 1h — chamado pela task `send_reminders`.
pub async fn enviar_lembretes_pendentes(
    db: &DatabaseConnection,
    provider: &Arc<dyn NotificationProvider>,
    agora: DateTime<Utc>,
) -> usize {
    let mut enviados = 0;

    let janelas = [
        (
            crate::models::appointments::Lembrete::VinteQuatroHoras,
            agora + chrono::Duration::minutes(23 * 60 + 50),
            agora + chrono::Duration::minutes(24 * 60 + 10),
        ),
        (
            crate::models::appointments::Lembrete::UmaHora,
            agora + chrono::Duration::minutes(50),
            agora + chrono::Duration::minutes(70),
        ),
    ];

    for (tipo, inicio, fim) in janelas {
        let pendentes =
            match AppointmentModel::find_aguardando_lembrete(db, tipo, inicio, fim).await {
                Ok(lista) => lista,
                Err(e) => {
                    tracing::error!(error = %e, "falha ao buscar lembretes pendentes");
                    continue;
                }
            };

        for appt in pendentes {
            let resultado = enviar_para_agendamento(db, &appt, provider, |dados| {
                if matches!(
                    tipo,
                    crate::models::appointments::Lembrete::VinteQuatroHoras
                ) {
                    mensagem_lembrete_24h(
                        &appt.client_name,
                        &dados.estabelecimento,
                        &dados.servico,
                        &dados.data_hora,
                    )
                } else {
                    mensagem_lembrete_1h(
                        &appt.client_name,
                        &dados.estabelecimento,
                        &dados.servico,
                        &dados.data_hora,
                    )
                }
            })
            .await;

            match resultado {
                Ok(()) => {
                    if let Err(e) = AppointmentModel::marcar_lembrete(db, appt.id, tipo).await {
                        tracing::warn!(appointment_id = %appt.id, error = %e, "falha ao marcar lembrete");
                    }
                    enviados += 1;
                }
                Err(e) => {
                    tracing::warn!(appointment_id = %appt.id, error = %e, "falha ao enviar lembrete");
                }
            }
        }
    }

    enviados
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_telefone_para_formato_internacional() {
        assert_eq!(
            normalizar_telefone("11987654321").as_deref(),
            Some("5511987654321")
        );
        assert_eq!(
            normalizar_telefone("(11) 98765-4321").as_deref(),
            Some("5511987654321")
        );
        assert_eq!(
            normalizar_telefone("5511987654321").as_deref(),
            Some("5511987654321")
        );
        assert_eq!(normalizar_telefone("12345").as_deref(), None);
    }

    #[test]
    fn formata_data_hora_no_fuso_do_tenant() {
        let instante = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 10, 19, 13, 0, 0).unwrap();
        assert_eq!(
            formatar_data_hora("America/Sao_Paulo", instante),
            "19/10/2026 às 10:00"
        );
    }

    #[test]
    fn templates_incluem_dados_do_agendamento() {
        let msg = mensagem_confirmacao(
            "Maria",
            "Barbearia do Zéu",
            "Corte",
            "José",
            "19/10/2026 às 10:00",
        );
        assert!(msg.contains("Maria"));
        assert!(msg.contains("Barbearia do Zéu"));
        assert!(msg.contains("19/10/2026 às 10:00"));

        let lembrete = mensagem_lembrete_24h("Maria", "Barbearia", "Corte", "20/10/2026 às 10:00");
        assert!(lembrete.contains("Lembrete"));

        let uma_hora = mensagem_lembrete_1h("Maria", "Barbearia", "Corte", "20/10/2026 às 10:00");
        assert!(uma_hora.contains("1 hora"));
    }
}
