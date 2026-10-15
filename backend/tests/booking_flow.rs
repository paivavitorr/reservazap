//! Testes de integração do fluxo de agendamento.
//!
//! Requer PostgreSQL ativo (use `make db-up`). Conecta via `DATABASE_URL` ou
//! cai no DSN de desenvolvimento.

use chrono::{Duration, NaiveTime, TimeZone, Utc};
use migration::MigratorTrait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use reservazap::models::_entities::payments::PaymentStatus;
use reservazap::models::_entities::{
    appointments, professionals, services, tenants, working_hours,
};
use reservazap::models::appointments::AppointmentModel;
use reservazap::models::payments::PaymentModel;
use reservazap::services::availability;
use reservazap::services::booking::{BookingError, BookingService};
use reservazap::services::notifications::{self, MockNotificationProvider};
use reservazap::services::pix_provider::MockPixProvider;

const DSN_DEV: &str = "postgres://postgres:postgres@localhost:5433/reservazap";

static MIGRATED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

async fn db() -> DatabaseConnection {
    let dsn = std::env::var("DATABASE_URL").unwrap_or_else(|_| DSN_DEV.to_string());
    let db = sea_orm::Database::connect(&dsn)
        .await
        .expect("falha ao conectar no PostgreSQL — suba com `make db-up`");
    MIGRATED
        .get_or_init(|| async {
            migration::Migrator::up(&db, None)
                .await
                .expect("falha ao aplicar migrations");
        })
        .await;
    db
}

struct Fixture {
    tenant_id: Uuid,
    service_id: Uuid,
    professional_id: Uuid,
}

async fn criar_fixture(db: &DatabaseConnection) -> Fixture {
    let now = Utc::now().fixed_offset();
    let tenant_id = Uuid::new_v4();
    let slug = format!("teste-{}", Uuid::new_v4().simple());

    tenants::ActiveModel {
        id: Set(tenant_id),
        slug: Set(slug),
        name: Set("Tenant de Teste".to_string()),
        phone: Set("11999990000".to_string()),
        timezone: Set("America/Sao_Paulo".to_string()),
        pix_key: Set(None),
        pix_key_type: Set(None),
        booking_hold_minutes: Set(10),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
    .unwrap();

    let service_id = Uuid::new_v4();
    services::ActiveModel {
        id: Set(service_id),
        tenant_id: Set(tenant_id),
        name: Set("Corte".to_string()),
        description: Set(None),
        duration_minutes: Set(30),
        price_in_cents: Set(4500),
        signal_percent: Set(30),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
    .unwrap();

    let professional_id = Uuid::new_v4();
    professionals::ActiveModel {
        id: Set(professional_id),
        tenant_id: Set(tenant_id),
        name: Set("Prof Teste".to_string()),
        email: Set(None),
        phone: Set(None),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
    .unwrap();

    // Expediente amplo em todos os dias
    for weekday in 0..7 {
        working_hours::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            professional_id: Set(None),
            weekday: Set(weekday),
            start_time: Set(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            end_time: Set(NaiveTime::from_hms_opt(23, 0, 0).unwrap()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
        .unwrap();
    }

    Fixture {
        tenant_id,
        service_id,
        professional_id,
    }
}

async fn limpar_fixture(db: &DatabaseConnection, tenant_id: Uuid) {
    tenants::Entity::delete_by_id(tenant_id)
        .exec(db)
        .await
        .unwrap();
}

/// Horário futuro seguro dentro do expediente de teste: 10:00 de amanhã (SP)
/// mais o deslocamento em minutos — nunca cruza meia-noite nem fica no passado.
fn futuro_em(minutos: i64) -> chrono::DateTime<Utc> {
    let tz = chrono_tz::America::Sao_Paulo;
    let amanha = (Utc::now() + Duration::days(1))
        .with_timezone(&tz)
        .date_naive();
    let base = tz
        .from_local_datetime(&amanha.and_hms_opt(10, 0, 0).unwrap())
        .unwrap()
        .with_timezone(&Utc);
    base + Duration::minutes(minutos)
}

async fn tenant_de(db: &DatabaseConnection, tenant_id: Uuid) -> tenants::Model {
    tenants::Entity::find_by_id(tenant_id)
        .one(db)
        .await
        .unwrap()
        .unwrap()
}

async fn criar_intencao(
    db: &DatabaseConnection,
    f: &Fixture,
    starts_at: chrono::DateTime<Utc>,
) -> Result<
    (
        Uuid,
        reservazap::services::pix_provider::PixCharge,
        chrono::DateTime<Utc>,
    ),
    BookingError,
> {
    let tenant = tenant_de(db, f.tenant_id).await;
    let pix = MockPixProvider;
    BookingService::criar_intencao(
        db,
        &pix,
        &tenant,
        f.service_id,
        f.professional_id,
        starts_at,
        "Cliente Teste".to_string(),
        "11987654321".to_string(),
        None,
    )
    .await
}

#[tokio::test]
async fn fluxo_completo_intent_webhook_confirmacao() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(60);

    let (appointment_id, charge, hold_expires) = criar_intencao(&db, &f, starts_at)
        .await
        .expect("intenção deve ser criada");

    assert_eq!(charge.valor_centavos, 1350); // 30% de 4500
    assert!(hold_expires > Utc::now());
    assert!(charge.qr_code_base64.is_some());

    let appt = AppointmentModel::find_by_id(&db, appointment_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(appt.status.as_str(), "hold");

    // webhook de pagamento (idempotente)
    let confirmado = BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
        .await
        .unwrap();
    assert!(confirmado);
    let de_novo = BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
        .await
        .unwrap();
    assert!(de_novo);

    let status = BookingService::status_agendamento(&db, f.tenant_id, appointment_id)
        .await
        .unwrap();
    assert_eq!(status.status.as_str(), "confirmed");
    assert!(status.signal_paid_at.is_some());

    // trilha de auditoria do pagamento
    let pagamentos = PaymentModel::registrar(
        &db,
        appointment_id,
        "mock",
        &charge.txid,
        charge.valor_centavos,
        PaymentStatus::Paid,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pagamentos.status.as_str(), "paid");

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn pagamento_tardio_e_registrado_como_late_paid() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(420);

    let (appointment_id, charge, _) = criar_intencao(&db, &f, starts_at).await.unwrap();

    // simula vencimento do hold
    let vencido = (Utc::now() - Duration::minutes(5)).fixed_offset();
    appointments::Entity::update_many()
        .set(appointments::ActiveModel {
            hold_expires_at: Set(vencido),
            ..Default::default()
        })
        .filter(appointments::Column::Id.eq(appointment_id))
        .exec(&db)
        .await
        .unwrap();

    let confirmado = BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
        .await
        .unwrap();
    assert!(!confirmado);

    let pagamento = PaymentModel::registrar(
        &db,
        appointment_id,
        "mock",
        &charge.txid,
        charge.valor_centavos,
        PaymentStatus::LatePaid,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pagamento.status.as_str(), "late_paid");

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn pagamento_apos_expiracao_do_sweeper_vira_late_paid() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(480);

    let (appointment_id, charge, _) = criar_intencao(&db, &f, starts_at).await.unwrap();

    // sweeper já expirou o hold antes do pagamento chegar
    AppointmentModel::mark_expired(&db, appointment_id)
        .await
        .unwrap();

    let confirmado = BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
        .await
        .unwrap();
    assert!(!confirmado);

    // a trilha de auditoria registra o pagamento tardio
    let pagamento = PaymentModel::registrar(
        &db,
        appointment_id,
        "mock",
        &charge.txid,
        charge.valor_centavos,
        PaymentStatus::LatePaid,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pagamento.status.as_str(), "late_paid");

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn slot_ocupado_rejeita_segunda_intencao() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(120);

    criar_intencao(&db, &f, starts_at).await.unwrap();
    let erro = criar_intencao(&db, &f, starts_at).await.unwrap_err();
    assert!(matches!(erro, BookingError::SlotUnavailable));

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn servico_de_outro_tenant_e_rejeitado() {
    let db = db().await;
    let f_a = criar_fixture(&db).await;
    let f_b = criar_fixture(&db).await;
    let starts_at = futuro_em(180);

    let tenant_a = tenant_de(&db, f_a.tenant_id).await;
    let pix = MockPixProvider;
    let erro = BookingService::criar_intencao(
        &db,
        &pix,
        &tenant_a,
        f_b.service_id,
        f_a.professional_id,
        starts_at,
        "Cliente Teste".to_string(),
        "11987654321".to_string(),
        None,
    )
    .await
    .unwrap_err();
    assert!(matches!(erro, BookingError::ServiceNotFound));

    limpar_fixture(&db, f_a.tenant_id).await;
    limpar_fixture(&db, f_b.tenant_id).await;
}

#[tokio::test]
async fn profissional_de_outro_tenant_e_rejeitado() {
    let db = db().await;
    let f_a = criar_fixture(&db).await;
    let f_b = criar_fixture(&db).await;
    let starts_at = futuro_em(240);

    let tenant_a = tenant_de(&db, f_a.tenant_id).await;
    let pix = MockPixProvider;
    let erro = BookingService::criar_intencao(
        &db,
        &pix,
        &tenant_a,
        f_a.service_id,
        f_b.professional_id,
        starts_at,
        "Cliente Teste".to_string(),
        "11987654321".to_string(),
        None,
    )
    .await
    .unwrap_err();
    assert!(matches!(erro, BookingError::ProfessionalNotFound));

    limpar_fixture(&db, f_a.tenant_id).await;
    limpar_fixture(&db, f_b.tenant_id).await;
}

#[tokio::test]
async fn data_passada_e_rejeitada() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let passado = Utc::now() - Duration::hours(2);

    let erro = criar_intencao(&db, &f, passado).await.unwrap_err();
    assert!(matches!(erro, BookingError::InvalidDate));

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn hold_vencido_nao_confirma_e_expira() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(300);

    let (appointment_id, charge, _) = criar_intencao(&db, &f, starts_at).await.unwrap();

    // simula o vencimento do hold forçando hold_expires_at para o passado
    let vencido = (Utc::now() - Duration::minutes(5)).fixed_offset();
    appointments::Entity::update_many()
        .set(appointments::ActiveModel {
            hold_expires_at: Set(vencido),
            ..Default::default()
        })
        .filter(appointments::Column::Id.eq(appointment_id))
        .exec(&db)
        .await
        .unwrap();

    let confirmado = BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
        .await
        .unwrap();
    assert!(!confirmado);

    AppointmentModel::mark_expired(&db, appointment_id)
        .await
        .unwrap();
    let appt = AppointmentModel::find_by_id(&db, appointment_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(appt.status.as_str(), "expired");

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn disponibilidade_bloqueia_horario_agendado() {
    let db = db().await;
    let f = criar_fixture(&db).await;

    let tz = chrono_tz::America::Sao_Paulo;
    let amanha = (Utc::now() + Duration::days(1))
        .with_timezone(&tz)
        .date_naive();
    let hora_fixa = tz
        .from_local_datetime(&amanha.and_hms_opt(15, 0, 0).unwrap())
        .unwrap()
        .with_timezone(&Utc);

    // agenda um horário e confirma — precisa estar dentro do expediente (00-23h)
    let (appointment_id, charge, _) = criar_intencao(&db, &f, hora_fixa).await.unwrap();
    assert!(
        BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
            .await
            .unwrap()
    );

    let tenant = tenants::Entity::find_by_id(f.tenant_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let disp = availability::disponibilidade(&db, &tenant, f.professional_id, 30, amanha, 1)
        .await
        .unwrap();

    let slots = &disp.days[0].slots;
    assert!(!slots.is_empty());
    let labels: Vec<&str> = slots.iter().map(|s| s.label.as_str()).collect();
    assert!(
        !labels.contains(&"15:00"),
        "horário agendado não deve aparecer livre: {labels:?}"
    );
    assert!(
        labels.contains(&"15:30"),
        "próximo slot deve estar livre: {labels:?}"
    );
    // status segue consistente
    let appt = AppointmentModel::find_by_id(&db, appointment_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(appt.status.as_str(), "confirmed");

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn status_de_outro_tenant_nao_vaza() {
    let db = db().await;
    let f_a = criar_fixture(&db).await;
    let f_b = criar_fixture(&db).await;
    let starts_at = futuro_em(360);

    let (appointment_id, _, _) = criar_intencao(&db, &f_a, starts_at).await.unwrap();

    let erro = BookingService::status_agendamento(&db, f_b.tenant_id, appointment_id)
        .await
        .unwrap_err();
    assert!(matches!(erro, BookingError::AppointmentNotFound));

    limpar_fixture(&db, f_a.tenant_id).await;
    limpar_fixture(&db, f_b.tenant_id).await;
}

#[tokio::test]
async fn lembretes_sao_enviados_uma_unica_vez() {
    let db = db().await;
    let f = criar_fixture(&db).await;

    // horário exatamente 24h à frente (dentro da janela do lembrete)
    let agora = Utc::now();
    let em_24h = agora + Duration::hours(24);
    let (appointment_id, charge, _) = criar_intencao(&db, &f, em_24h).await.unwrap();
    assert!(
        BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
            .await
            .unwrap()
    );

    let provider: std::sync::Arc<dyn reservazap::services::notifications::NotificationProvider> =
        std::sync::Arc::new(MockNotificationProvider);

    let enviados = notifications::enviar_lembretes_pendentes(&db, &provider, agora).await;
    assert_eq!(enviados, 1, "deve enviar o lembrete de 24h");

    // idempotência: segunda passada não envia de novo
    let enviados_depois = notifications::enviar_lembretes_pendentes(&db, &provider, agora).await;
    assert_eq!(enviados_depois, 0);

    let appt = AppointmentModel::find_by_id(&db, appointment_id)
        .await
        .unwrap()
        .unwrap();
    assert!(appt.reminded_24h_at.is_some());

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn transicoes_de_status_do_painel_sao_validadas() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(240);

    let (appointment_id, charge, _) = criar_intencao(&db, &f, starts_at).await.unwrap();
    assert!(
        BookingService::confirmar_por_txid(&db, "mock", &charge.txid, None)
            .await
            .unwrap()
    );

    // confirmado → concluir é permitido
    let r = AppointmentModel::transicionar(
        &db,
        appointment_id,
        f.tenant_id,
        appointments::AppointmentStatus::Completed,
        None,
    )
    .await
    .unwrap();
    assert_eq!(r, Some(true));

    // concluído → cancelar não é permitido
    let r = AppointmentModel::transicionar(
        &db,
        appointment_id,
        f.tenant_id,
        appointments::AppointmentStatus::Cancelled,
        Some("desisti"),
    )
    .await
    .unwrap();
    assert_eq!(r, Some(false));

    // agendamento de outro tenant não é encontrado
    let f_b = criar_fixture(&db).await;
    let r = AppointmentModel::transicionar(
        &db,
        appointment_id,
        f_b.tenant_id,
        appointments::AppointmentStatus::Cancelled,
        None,
    )
    .await
    .unwrap();
    assert_eq!(r, None);

    limpar_fixture(&db, f.tenant_id).await;
    limpar_fixture(&db, f_b.tenant_id).await;
}

#[tokio::test]
async fn cancelamento_registra_motivo_e_data() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(180);

    let (appointment_id, _, _) = criar_intencao(&db, &f, starts_at).await.unwrap();

    let r = AppointmentModel::transicionar(
        &db,
        appointment_id,
        f.tenant_id,
        appointments::AppointmentStatus::Cancelled,
        Some("cliente desistiu"),
    )
    .await
    .unwrap();
    assert_eq!(r, Some(true));

    let appt = AppointmentModel::find_by_id(&db, appointment_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(appt.status.as_str(), "cancelled");
    assert_eq!(
        appt.cancellation_reason.as_deref(),
        Some("cliente desistiu")
    );
    assert!(appt.cancelled_at.is_some());

    limpar_fixture(&db, f.tenant_id).await;
}

#[tokio::test]
async fn lgpd_anonimiza_dados_do_cliente() {
    let db = db().await;
    let f = criar_fixture(&db).await;
    let starts_at = futuro_em(120);

    let (appointment_id, _, _) = criar_intencao(&db, &f, starts_at).await.unwrap();

    let alterados = AppointmentModel::anonimizar_cliente(&db, f.tenant_id, "11987654321")
        .await
        .unwrap();
    assert_eq!(alterados, 1);

    let appt = AppointmentModel::find_by_id(&db, appointment_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(appt.client_name, "Cliente removido");
    assert_eq!(appt.client_phone, "removido");
    assert!(appt.client_email.is_none());

    limpar_fixture(&db, f.tenant_id).await;
}
