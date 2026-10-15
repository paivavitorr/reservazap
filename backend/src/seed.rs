use chrono::{NaiveTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::models::_entities::{professionals, services, tenants, working_hours};

const SLUG_DEMO: &str = "barbearia-do-zeu";

/// Cria o estabelecimento de demonstração com serviços, profissionais e
/// expediente. Idempotente: não duplica se o slug já existir.
pub async fn seed_demo(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let existente = tenants::Entity::find()
        .filter(tenants::Column::Slug.eq(SLUG_DEMO))
        .one(db)
        .await?;
    if existente.is_some() {
        tracing::info!(slug = SLUG_DEMO, "seed: demo já existe, nada a fazer");
        return Ok(());
    }

    let now = Utc::now().fixed_offset();
    let tenant_id = Uuid::new_v4();

    tenants::ActiveModel {
        id: Set(tenant_id),
        slug: Set(SLUG_DEMO.to_string()),
        name: Set("Barbearia do Zéu".to_string()),
        phone: Set("11999990000".to_string()),
        timezone: Set("America/Sao_Paulo".to_string()),
        pix_key: Set(Some("reservazap@mock.pix".to_string())),
        pix_key_type: Set(Some("email".to_string())),
        booking_hold_minutes: Set(10),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;

    let servicos = [
        (
            "Corte de cabelo",
            "Corte tradicional com máquina e tesoura",
            30,
            4500i64,
        ),
        ("Barba", "Barba desenhada com toalha quente", 20, 3000),
        ("Corte + Barba", "Combo completo", 50, 6500),
    ];
    for (nome, descricao, duracao, preco) in servicos {
        services::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(nome.to_string()),
            description: Set(Some(descricao.to_string())),
            duration_minutes: Set(duracao),
            price_in_cents: Set(preco),
            signal_percent: Set(30),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;
    }

    for nome in ["José Carlos (Zéu)", "Ana Beatriz"] {
        professionals::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(nome.to_string()),
            email: Set(None),
            phone: Set(None),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;
    }

    let t = |h, m| NaiveTime::from_hms_opt(h, m, 0).expect("horário válido");
    // weekday: 0=segunda .. 6=domingo
    let expediente: Vec<(i16, Vec<(NaiveTime, NaiveTime)>)> = vec![
        (0, vec![(t(9, 0), t(12, 0)), (t(13, 0), t(18, 0))]),
        (1, vec![(t(9, 0), t(12, 0)), (t(13, 0), t(18, 0))]),
        (2, vec![(t(9, 0), t(12, 0)), (t(13, 0), t(18, 0))]),
        (3, vec![(t(9, 0), t(12, 0)), (t(13, 0), t(18, 0))]),
        (4, vec![(t(9, 0), t(12, 0)), (t(13, 0), t(18, 0))]),
        (5, vec![(t(9, 0), t(13, 0))]),
    ];
    for (weekday, faixas) in expediente {
        for (start, end) in faixas {
            working_hours::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                professional_id: Set(None),
                weekday: Set(weekday),
                start_time: Set(start),
                end_time: Set(end),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(db)
            .await?;
        }
    }

    tracing::info!(slug = SLUG_DEMO, "seed: estabelecimento demo criado");
    Ok(())
}
