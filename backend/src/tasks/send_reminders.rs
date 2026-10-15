use loco_rs::prelude::*;

use crate::services::notifications;

pub struct SendReminders;

#[async_trait]
impl Task for SendReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "send_reminders".to_string(),
            detail: "Envia lembretes de WhatsApp 24h e 1h antes do atendimento".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let provider = notifications::from_env();
        let enviados = notifications::enviar_lembretes_pendentes(
            &app_context.db,
            &provider,
            chrono::Utc::now(),
        )
        .await;
        tracing::info!(enviados, "send_reminders concluído");
        Ok(())
    }
}
