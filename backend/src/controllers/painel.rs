use axum::routing::{get, patch, post};
use chrono::{DateTime, NaiveTime, Utc};
use loco_rs::controller::extractor::auth;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controllers::auth::usuario_do_token;
use crate::models::_entities::{appointments, professionals, services, working_hours};
use crate::models::appointments::AppointmentModel;
use crate::models::professionals::ProfessionalModel;
use crate::models::services::ServiceModel;

// ─────────────────────────── DTOs ───────────────────────────

#[derive(Debug, Deserialize)]
pub struct ServicoPayload {
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price_in_cents: i64,
    pub signal_percent: i16,
}

#[derive(Debug, Deserialize)]
pub struct ProfissionalPayload {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExpedienteItem {
    pub weekday: i16,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpedientePayload {
    pub items: Vec<ExpedienteItem>,
}

#[derive(Debug, Deserialize)]
pub struct CancelarPayload {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServicoInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price_in_cents: i64,
    pub signal_percent: i16,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ProfissionalInfo {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ExpedienteInfo {
    pub weekday: i16,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct AgendamentoInfo {
    pub id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub client_name: String,
    pub client_phone: String,
    pub status: String,
    pub service: String,
    pub professional: String,
    pub signal_amount_in_cents: i64,
    pub signal_paid_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResumoInfo {
    pub hoje: usize,
    pub semana: usize,
    pub confirmados_hoje: usize,
    pub taxa_no_show_30d: f64,
    pub sinais_pagos_30d: i64,
}

// ─────────────────────────── Serviços ───────────────────────────

async fn listar_servicos(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
) -> Result<Json<Vec<ServicoInfo>>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let itens = ServiceModel::find_by_tenant_all(&ctx.db, user.tenant_id).await?;
    Ok(Json(itens.into_iter().map(servico_info).collect()))
}

async fn criar_servico(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Json(payload): Json<ServicoPayload>,
) -> Result<Json<ServicoInfo>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    validar_servico(&payload)?;

    let now = Utc::now().fixed_offset();
    let model = services::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(user.tenant_id),
        name: Set(payload.name.trim().to_string()),
        description: Set(payload.description.map(|d| d.trim().to_string())),
        duration_minutes: Set(payload.duration_minutes),
        price_in_cents: Set(payload.price_in_cents),
        signal_percent: Set(payload.signal_percent),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&ctx.db)
    .await?;
    Ok(Json(servico_info(model)))
}

async fn atualizar_servico(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
    Json(payload): Json<ServicoPayload>,
) -> Result<Json<ServicoInfo>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    validar_servico(&payload)?;

    let existente = ServiceModel::find_by_id_for_tenant(&ctx.db, user.tenant_id, id)
        .await?
        .ok_or(Error::NotFound)?;

    let mut active: services::ActiveModel = existente.into();
    active.name = Set(payload.name.trim().to_string());
    active.description = Set(payload.description.map(|d| d.trim().to_string()));
    active.duration_minutes = Set(payload.duration_minutes);
    active.price_in_cents = Set(payload.price_in_cents);
    active.signal_percent = Set(payload.signal_percent);
    active.updated_at = Set(Utc::now().fixed_offset());
    let model = active.update(&ctx.db).await?;
    Ok(Json(servico_info(model)))
}

async fn desativar_servico(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let existente = ServiceModel::find_by_id_for_tenant(&ctx.db, user.tenant_id, id)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: services::ActiveModel = existente.into();
    active.is_active = Set(false);
    active.updated_at = Set(Utc::now().fixed_offset());
    active.update(&ctx.db).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn validar_servico(p: &ServicoPayload) -> Result<()> {
    if p.name.trim().is_empty() || p.name.len() > 120 {
        return Err(Error::BadRequest("nome do serviço inválido".into()));
    }
    if p.duration_minutes <= 0 || p.duration_minutes > 24 * 60 {
        return Err(Error::BadRequest("duração inválida".into()));
    }
    if p.price_in_cents < 0 {
        return Err(Error::BadRequest("preço inválido".into()));
    }
    if !(0..=100).contains(&p.signal_percent) {
        return Err(Error::BadRequest(
            "percentual do sinal deve ser 0–100".into(),
        ));
    }
    Ok(())
}

fn servico_info(m: services::Model) -> ServicoInfo {
    ServicoInfo {
        id: m.id,
        name: m.name,
        description: m.description,
        duration_minutes: m.duration_minutes,
        price_in_cents: m.price_in_cents,
        signal_percent: m.signal_percent,
        is_active: m.is_active,
    }
}

// ─────────────────────────── Profissionais ───────────────────────────

async fn listar_profissionais(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
) -> Result<Json<Vec<ProfissionalInfo>>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let itens = ProfessionalModel::find_by_tenant_all(&ctx.db, user.tenant_id).await?;
    Ok(Json(itens.into_iter().map(profissional_info).collect()))
}

async fn criar_profissional(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Json(payload): Json<ProfissionalPayload>,
) -> Result<Json<ProfissionalInfo>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    if payload.name.trim().is_empty() || payload.name.len() > 120 {
        return Err(Error::BadRequest("nome do profissional inválido".into()));
    }

    let now = Utc::now().fixed_offset();
    let model = professionals::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(user.tenant_id),
        name: Set(payload.name.trim().to_string()),
        email: Set(payload.email),
        phone: Set(payload.phone),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&ctx.db)
    .await?;
    Ok(Json(profissional_info(model)))
}

async fn atualizar_profissional(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
    Json(payload): Json<ProfissionalPayload>,
) -> Result<Json<ProfissionalInfo>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    if payload.name.trim().is_empty() || payload.name.len() > 120 {
        return Err(Error::BadRequest("nome do profissional inválido".into()));
    }
    let existente = ProfessionalModel::find_by_id_for_tenant(&ctx.db, user.tenant_id, id)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: professionals::ActiveModel = existente.into();
    active.name = Set(payload.name.trim().to_string());
    active.email = Set(payload.email);
    active.phone = Set(payload.phone);
    active.updated_at = Set(Utc::now().fixed_offset());
    let model = active.update(&ctx.db).await?;
    Ok(Json(profissional_info(model)))
}

async fn desativar_profissional(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let existente = ProfessionalModel::find_by_id_for_tenant(&ctx.db, user.tenant_id, id)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: professionals::ActiveModel = existente.into();
    active.is_active = Set(false);
    active.updated_at = Set(Utc::now().fixed_offset());
    active.update(&ctx.db).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn profissional_info(m: professionals::Model) -> ProfissionalInfo {
    ProfissionalInfo {
        id: m.id,
        name: m.name,
        email: m.email,
        phone: m.phone,
        is_active: m.is_active,
    }
}

// ─────────────────────────── Expediente ───────────────────────────

async fn listar_expediente(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
) -> Result<Json<Vec<ExpedienteInfo>>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let rows = working_hours::Entity::find()
        .filter(working_hours::Column::TenantId.eq(user.tenant_id))
        .filter(working_hours::Column::ProfessionalId.is_null())
        .order_by_asc(working_hours::Column::Weekday)
        .order_by_asc(working_hours::Column::StartTime)
        .all(&ctx.db)
        .await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| ExpedienteInfo {
                weekday: r.weekday,
                start: r.start_time.format("%H:%M").to_string(),
                end: r.end_time.format("%H:%M").to_string(),
            })
            .collect(),
    ))
}

async fn salvar_expediente(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Json(payload): Json<ExpedientePayload>,
) -> Result<Json<Vec<ExpedienteInfo>>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    if payload.items.len() > 7 * 4 {
        return Err(Error::BadRequest("expediente com faixas demais".into()));
    }

    let mut faixas = Vec::with_capacity(payload.items.len());
    for item in &payload.items {
        if !(0..=6).contains(&item.weekday) {
            return Err(Error::BadRequest("dia da semana inválido".into()));
        }
        let start = parse_hora(&item.start)?;
        let end = parse_hora(&item.end)?;
        if end <= start {
            return Err(Error::BadRequest(
                "fim do expediente deve ser depois do início".into(),
            ));
        }
        faixas.push((item.weekday, start, end));
    }

    // faixas do mesmo dia não podem se sobrepor
    let mut ordenadas = faixas.clone();
    ordenadas.sort_by_key(|(weekday, start, _)| (*weekday, *start));
    for janela in ordenadas.windows(2) {
        let (dia_a, _, fim_a) = janela[0];
        let (dia_b, inicio_b, _) = janela[1];
        if dia_a == dia_b && inicio_b < fim_a {
            return Err(Error::BadRequest(
                "faixas de horário sobrepostas no mesmo dia".to_string(),
            ));
        }
    }

    use sea_orm::TransactionTrait;
    let txn = ctx
        .db
        .begin()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    // substitui todo o expediente geral do tenant
    working_hours::Entity::delete_many()
        .filter(working_hours::Column::TenantId.eq(user.tenant_id))
        .filter(working_hours::Column::ProfessionalId.is_null())
        .exec(&txn)
        .await?;

    let now = Utc::now().fixed_offset();
    for (weekday, start, end) in &faixas {
        working_hours::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(user.tenant_id),
            professional_id: Set(None),
            weekday: Set(*weekday),
            start_time: Set(*start),
            end_time: Set(*end),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&txn)
        .await?;
    }
    txn.commit()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    listar_expediente(State(ctx), auth).await
}

fn parse_hora(raw: &str) -> Result<NaiveTime> {
    NaiveTime::parse_from_str(raw, "%H:%M")
        .map_err(|_| Error::BadRequest(format!("horário inválido: {raw}")))
}

// ─────────────────────────── Agenda e ações ───────────────────────────

#[derive(Debug, Deserialize)]
pub struct AgendaQuery {
    pub de: Option<String>,
    pub ate: Option<String>,
}

async fn listar_agenda(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Query(query): Query<AgendaQuery>,
) -> Result<Json<Vec<AgendamentoInfo>>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let de = parse_data(query.de.as_deref(), 0)?;
    let ate = parse_data_fim(query.ate.as_deref())?;

    let rows = AppointmentModel::find_by_tenant_range(&ctx.db, user.tenant_id, de, ate).await?;
    let servicos = ServiceModel::find_by_tenant_all(&ctx.db, user.tenant_id).await?;
    let profissionais = ProfessionalModel::find_by_tenant_all(&ctx.db, user.tenant_id).await?;

    let nome_servico = |id: Uuid| {
        servicos
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Serviço".to_string())
    };
    let nome_prof = |id: Uuid| {
        profissionais
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Profissional".to_string())
    };

    Ok(Json(
        rows.into_iter()
            .map(|a| AgendamentoInfo {
                id: a.id,
                starts_at: a.starts_at.with_timezone(&Utc),
                ends_at: a.ends_at.with_timezone(&Utc),
                client_name: a.client_name,
                client_phone: a.client_phone,
                status: a.status.as_str().to_string(),
                service: nome_servico(a.service_id),
                professional: nome_prof(a.professional_id),
                signal_amount_in_cents: a.signal_amount_in_cents,
                signal_paid_at: a.signal_paid_at.map(|dt| dt.with_timezone(&Utc)),
                cancellation_reason: a.cancellation_reason,
            })
            .collect(),
    ))
}

async fn cancelar_agendamento(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelarPayload>,
) -> Result<Json<serde_json::Value>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    aplicar_transicao(
        &ctx,
        user.tenant_id,
        id,
        appointments::AppointmentStatus::Cancelled,
        payload.reason.as_deref(),
    )
    .await
}

async fn concluir_agendamento(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    aplicar_transicao(
        &ctx,
        user.tenant_id,
        id,
        appointments::AppointmentStatus::Completed,
        None,
    )
    .await
}

async fn no_show_agendamento(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    aplicar_transicao(
        &ctx,
        user.tenant_id,
        id,
        appointments::AppointmentStatus::NoShow,
        None,
    )
    .await
}

async fn aplicar_transicao(
    ctx: &AppContext,
    tenant_id: Uuid,
    id: Uuid,
    proximo: appointments::AppointmentStatus,
    motivo: Option<&str>,
) -> Result<Json<serde_json::Value>> {
    match AppointmentModel::transicionar(&ctx.db, id, tenant_id, proximo, motivo).await? {
        None => Err(Error::NotFound),
        Some(true) => Ok(Json(serde_json::json!({ "ok": true }))),
        Some(false) => Err(Error::BadRequest(
            "transição de status não permitida neste momento".to_string(),
        )),
    }
}

async fn resumo(State(ctx): State<AppContext>, auth: auth::JWT) -> Result<Json<ResumoInfo>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    let agora = Utc::now();
    let inicio_hoje = agora.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let inicio_semana = inicio_hoje - chrono::Duration::days(7);
    let inicio_30d = inicio_hoje - chrono::Duration::days(30);

    let todos = AppointmentModel::find_by_tenant_range(
        &ctx.db,
        user.tenant_id,
        inicio_30d.fixed_offset(),
        (agora + chrono::Duration::days(1)).fixed_offset(),
    )
    .await?;

    let eh_hoje =
        |dt: DateTime<Utc>| dt >= inicio_hoje && dt < inicio_hoje + chrono::Duration::days(1);
    let hoje = todos
        .iter()
        .filter(|a| eh_hoje(a.starts_at.with_timezone(&Utc)))
        .count();
    let confirmados_hoje = todos
        .iter()
        .filter(|a| {
            eh_hoje(a.starts_at.with_timezone(&Utc))
                && a.status == appointments::AppointmentStatus::Confirmed
        })
        .count();
    let semana = todos
        .iter()
        .filter(|a| {
            let dt = a.starts_at.with_timezone(&Utc);
            dt >= inicio_semana && dt < inicio_hoje + chrono::Duration::days(1)
        })
        .count();

    let encerrados_30d = todos
        .iter()
        .filter(|a| {
            matches!(
                a.status,
                appointments::AppointmentStatus::Completed
                    | appointments::AppointmentStatus::NoShow
            )
        })
        .count();
    let no_shows = todos
        .iter()
        .filter(|a| a.status == appointments::AppointmentStatus::NoShow)
        .count();
    let taxa_no_show_30d = if encerrados_30d == 0 {
        0.0
    } else {
        (no_shows as f64 / encerrados_30d as f64) * 100.0
    };

    let sinais_pagos_30d = todos
        .iter()
        .filter(|a| a.signal_paid_at.is_some())
        .map(|a| a.signal_amount_in_cents)
        .sum();

    Ok(Json(ResumoInfo {
        hoje,
        semana,
        confirmados_hoje,
        taxa_no_show_30d: (taxa_no_show_30d * 10.0).round() / 10.0,
        sinais_pagos_30d,
    }))
}

fn parse_data(raw: Option<&str>, mais_dias: i64) -> Result<chrono::DateTime<chrono::FixedOffset>> {
    match raw {
        Some(s) => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| Error::BadRequest("data inválida (use YYYY-MM-DD)".to_string()))
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset()),
        None => Ok((Utc::now() + chrono::Duration::days(mais_dias)).fixed_offset()),
    }
}

/// Fim exclusivo do período: o dia informado em `ate` é incluído por inteiro.
fn parse_data_fim(raw: Option<&str>) -> Result<chrono::DateTime<chrono::FixedOffset>> {
    match raw {
        Some(s) => {
            let dia = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| Error::BadRequest("data inválida (use YYYY-MM-DD)".to_string()))?;
            let proximo = dia + chrono::Duration::days(1);
            Ok(proximo
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .fixed_offset())
        }
        None => Ok((Utc::now() + chrono::Duration::days(7)).fixed_offset()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("painel")
        .add("/servicos", get(listar_servicos).post(criar_servico))
        .add(
            "/servicos/{id}",
            patch(atualizar_servico).delete(desativar_servico),
        )
        .add(
            "/profissionais",
            get(listar_profissionais).post(criar_profissional),
        )
        .add(
            "/profissionais/{id}",
            patch(atualizar_profissional).delete(desativar_profissional),
        )
        .add("/expediente", get(listar_expediente).put(salvar_expediente))
        .add("/agenda", get(listar_agenda))
        .add("/agendamentos/{id}/cancelar", post(cancelar_agendamento))
        .add("/agendamentos/{id}/concluir", post(concluir_agendamento))
        .add("/agendamentos/{id}/no-show", post(no_show_agendamento))
        .add("/resumo", get(resumo))
}
