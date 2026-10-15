use crate::models::appointments::AppointmentModel;
use crate::services::notifications;
use loco_rs::prelude::*;

pub struct ExpireHolds;

#[async_trait]
impl Task for ExpireHolds {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "expire_holds".to_string(),
            detail: "Expira agendamentos com hold de Pix vencido e libera o horário".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let db = &app_context.db;
        let expired = AppointmentModel::find_expired_holds(db).await?;

        if expired.is_empty() {
            tracing::debug!("expire_holds: nenhum hold expirado encontrado");
            return Ok(());
        }

        let provider = notifications::from_env();
        let count = expired.len();
        for appt in expired {
            match AppointmentModel::mark_expired(db, appt.id).await {
                Ok(true) => {
                    notifications::notificar_horario_liberado(db, &appt, &provider).await;
                    tracing::info!(
                        appointment_id = %appt.id,
                        professional_id = %appt.professional_id,
                        starts_at = %appt.starts_at,
                        "hold expirado, horário liberado"
                    );
                }
                Ok(false) => {
                    tracing::debug!(
                        appointment_id = %appt.id,
                        "hold não expirado (transição concorrente)"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        appointment_id = %appt.id,
                        error = %e,
                        "falha ao marcar hold como expirado"
                    );
                }
            }
        }

        tracing::info!(count, "expire_holds concluído");
        Ok(())
    }
}
