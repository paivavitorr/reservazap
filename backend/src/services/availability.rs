use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use crate::models::_entities::tenants;
use crate::models::appointments::AppointmentModel;
use crate::models::working_hours::WorkingHoursModel;

/// Passo da grade de horários (minutos).
const SLOT_STEP_MINUTES: i64 = 30;
/// Máximo de dias consultados por vez.
const MAX_DAYS: i64 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaixaExpediente {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

/// Expediente semanal de um profissional (weekday 0=segunda .. 6=domingo).
/// Faixas específicas do profissional substituem as gerais do tenant.
#[derive(Debug, Clone, Default)]
pub struct Expediente {
    faixas: HashMap<i16, Vec<FaixaExpediente>>,
}

impl Expediente {
    pub fn faixas_do_dia(&self, weekday: i16) -> &[FaixaExpediente] {
        self.faixas.get(&weekday).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Slot {
    /// Instante UTC do início do horário.
    pub starts_at: DateTime<Utc>,
    /// Horário formatado no fuso do estabelecimento (ex.: "09:30").
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiaDisponivel {
    /// Data no fuso do estabelecimento (YYYY-MM-DD).
    pub date: String,
    pub weekday: String,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Disponibilidade {
    pub timezone: String,
    pub duration_minutes: i32,
    pub days: Vec<DiaDisponivel>,
}

#[derive(Debug, thiserror::Error)]
pub enum AvailabilityError {
    #[error("fuso horário inválido")]
    TimezoneInvalida,
    #[error("data inválida")]
    DataInvalida,
    #[error("erro de banco: {0}")]
    Database(#[from] sea_orm::DbErr),
}

/// Carrega o expediente semanal com uma única consulta (específicas + gerais).
pub async fn carregar_expediente(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    professional_id: Uuid,
) -> Result<Expediente, AvailabilityError> {
    let rows = WorkingHoursModel::find_do_profissional(db, tenant_id, professional_id).await?;

    let mut especificas: HashMap<i16, Vec<FaixaExpediente>> = HashMap::new();
    let mut gerais: HashMap<i16, Vec<FaixaExpediente>> = HashMap::new();
    for row in rows {
        let faixa = FaixaExpediente {
            start: row.start_time,
            end: row.end_time,
        };
        if row.professional_id.is_some() {
            especificas.entry(row.weekday).or_default().push(faixa);
        } else {
            gerais.entry(row.weekday).or_default().push(faixa);
        }
    }

    let mut faixas = gerais;
    for (weekday, faixas_pro) in especificas {
        faixas.insert(weekday, faixas_pro);
    }
    for lista in faixas.values_mut() {
        lista.sort_by_key(|f| f.start);
    }

    Ok(Expediente { faixas })
}

/// O intervalo `[starts_at, ends_at)` cabe inteiro em alguma faixa do expediente?
pub fn slot_cabe_no_expediente(
    expediente: &Expediente,
    tz: Tz,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
) -> bool {
    let inicio = starts_at.with_timezone(&tz);
    let fim = ends_at.with_timezone(&tz);
    if inicio.date_naive() != fim.date_naive() {
        return false;
    }

    let weekday = inicio.weekday().num_days_from_monday() as i16;
    let inicio_t = inicio.time();
    let fim_t = fim.time();

    expediente
        .faixas_do_dia(weekday)
        .iter()
        .any(|f| f.start <= inicio_t && fim_t <= f.end)
}

/// Gera os slots livres de um dia, em UTC, a partir das faixas de expediente
/// (interpretadas no fuso do tenant), da duração do serviço e dos intervalos
/// já ocupados.
pub fn gerar_slots_do_dia(
    tz: Tz,
    dia: NaiveDate,
    faixas: &[FaixaExpediente],
    duration_minutes: i32,
    ocupados: &[(DateTime<Utc>, DateTime<Utc>)],
    agora: DateTime<Utc>,
) -> Vec<Slot> {
    let duracao = Duration::minutes(i64::from(duration_minutes));
    let passo = Duration::minutes(SLOT_STEP_MINUTES);
    let mut slots = Vec::new();

    for faixa in faixas {
        let mut cursor = faixa.start;
        while cursor + Duration::minutes(i64::from(duration_minutes)) <= faixa.end {
            let inicio_local = match tz.from_local_datetime(&dia.and_time(cursor)) {
                chrono::LocalResult::Single(dt) => dt,
                chrono::LocalResult::Ambiguous(dt, _) => dt,
                chrono::LocalResult::None => {
                    cursor += passo;
                    continue;
                }
            };
            let inicio_utc = inicio_local.with_timezone(&Utc);
            let fim_utc = inicio_utc + duracao;

            let livre = inicio_utc >= agora
                && !ocupados
                    .iter()
                    .any(|(s, e)| inicio_utc < *e && fim_utc > *s);

            if livre {
                slots.push(Slot {
                    starts_at: inicio_utc,
                    label: cursor.format("%H:%M").to_string(),
                });
            }
            cursor += passo;
        }
    }

    slots.sort_by_key(|s| s.starts_at);
    slots
}

/// Disponibilidade dos próximos `days` dias a partir de `from` (fuso do tenant).
pub async fn disponibilidade(
    db: &DatabaseConnection,
    tenant: &tenants::Model,
    professional_id: Uuid,
    duration_minutes: i32,
    from: NaiveDate,
    days: i64,
) -> Result<Disponibilidade, AvailabilityError> {
    let tz = Tz::from_str(&tenant.timezone).map_err(|_| AvailabilityError::TimezoneInvalida)?;
    let days = days.clamp(1, MAX_DAYS);

    let primeiro_utc = data_para_utc(tz, from).ok_or(AvailabilityError::DataInvalida)?;
    let ultimo = from + Duration::days(days);
    let ultimo_utc = data_para_utc(tz, ultimo).ok_or(AvailabilityError::DataInvalida)?;

    let ocupados_model =
        AppointmentModel::find_active_overlapping(db, professional_id, primeiro_utc, ultimo_utc)
            .await?;
    let ocupados: Vec<(DateTime<Utc>, DateTime<Utc>)> = ocupados_model
        .iter()
        .map(|a| {
            (
                a.starts_at.with_timezone(&Utc),
                a.ends_at.with_timezone(&Utc),
            )
        })
        .collect();

    let expediente = carregar_expediente(db, tenant.id, professional_id).await?;
    let agora = Utc::now();
    let mut days_out = Vec::new();

    for offset in 0..days {
        let dia = from + Duration::days(offset);
        let weekday = dia.weekday().num_days_from_monday() as i16;
        let slots = gerar_slots_do_dia(
            tz,
            dia,
            expediente.faixas_do_dia(weekday),
            duration_minutes,
            &ocupados,
            agora,
        );

        days_out.push(DiaDisponivel {
            date: dia.format("%Y-%m-%d").to_string(),
            weekday: nome_weekday(weekday),
            slots,
        });
    }

    Ok(Disponibilidade {
        timezone: tenant.timezone.clone(),
        duration_minutes,
        days: days_out,
    })
}

fn data_para_utc(tz: Tz, data: NaiveDate) -> Option<DateTime<Utc>> {
    let inicio = data.and_hms_opt(0, 0, 0)?;
    match tz.from_local_datetime(&inicio) {
        chrono::LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(dt, _) => Some(dt.with_timezone(&Utc)),
        chrono::LocalResult::None => None,
    }
}

fn nome_weekday(weekday: i16) -> String {
    match weekday {
        0 => "segunda-feira",
        1 => "terça-feira",
        2 => "quarta-feira",
        3 => "quinta-feira",
        4 => "sexta-feira",
        5 => "sábado",
        _ => "domingo",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    #[test]
    fn gera_slots_dentro_do_expediente() {
        let tz: Tz = "America/Sao_Paulo".parse().unwrap();
        let dia = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(); // segunda
        let faixas = vec![FaixaExpediente {
            start: t(9, 0),
            end: t(10, 0),
        }];
        let agora = tz
            .with_ymd_and_hms(2026, 10, 19, 0, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let slots = gerar_slots_do_dia(tz, dia, &faixas, 30, &[], agora);
        let labels: Vec<_> = slots.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, vec!["09:00", "09:30"]);
    }

    #[test]
    fn descarta_slots_passados_e_ocupados() {
        let tz: Tz = "America/Sao_Paulo".parse().unwrap();
        let dia = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap();
        let faixas = vec![FaixaExpediente {
            start: t(9, 0),
            end: t(11, 0),
        }];
        // agora = 10:00 local → slots 09:xx caem fora
        let agora = tz
            .with_ymd_and_hms(2026, 10, 19, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        // ocupado: 10:30–11:00 local
        let occ_ini = tz
            .with_ymd_and_hms(2026, 10, 19, 10, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        let occ_fim = tz
            .with_ymd_and_hms(2026, 10, 19, 11, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let slots = gerar_slots_do_dia(tz, dia, &faixas, 30, &[(occ_ini, occ_fim)], agora);
        let labels: Vec<_> = slots.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, vec!["10:00"]);
    }

    #[test]
    fn slot_de_45_min_nao_ultrapassa_o_expediente() {
        let tz: Tz = "America/Sao_Paulo".parse().unwrap();
        let dia = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap();
        let faixas = vec![FaixaExpediente {
            start: t(9, 0),
            end: t(10, 0),
        }];
        let agora = tz
            .with_ymd_and_hms(2026, 10, 19, 0, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let slots = gerar_slots_do_dia(tz, dia, &faixas, 45, &[], agora);
        let labels: Vec<_> = slots.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, vec!["09:00"]); // 09:30 + 45min passaria das 10:00
    }

    #[test]
    fn slot_cabe_no_expediente_respeita_faixas() {
        let tz: Tz = "America/Sao_Paulo".parse().unwrap();
        let mut expediente = Expediente::default();
        expediente.faixas.insert(
            0, // segunda
            vec![FaixaExpediente {
                start: t(9, 0),
                end: t(12, 0),
            }],
        );

        let ini = tz
            .with_ymd_and_hms(2026, 10, 19, 9, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let fim = tz
            .with_ymd_and_hms(2026, 10, 19, 9, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert!(slot_cabe_no_expediente(&expediente, tz, ini, fim));

        // começa dentro, termina depois do expediente
        let fim_fora = tz
            .with_ymd_and_hms(2026, 10, 19, 12, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert!(!slot_cabe_no_expediente(&expediente, tz, ini, fim_fora));

        // fora de qualquer faixa
        let ini_tarde = tz
            .with_ymd_and_hms(2026, 10, 19, 15, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let fim_tarde = tz
            .with_ymd_and_hms(2026, 10, 19, 15, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert!(!slot_cabe_no_expediente(
            &expediente,
            tz,
            ini_tarde,
            fim_tarde
        ));

        // dia sem expediente (terça)
        let ini_terca = tz
            .with_ymd_and_hms(2026, 10, 20, 9, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let fim_terca = tz
            .with_ymd_and_hms(2026, 10, 20, 9, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert!(!slot_cabe_no_expediente(
            &expediente,
            tz,
            ini_terca,
            fim_terca
        ));
    }

    #[test]
    fn labels_de_weekday() {
        assert_eq!(nome_weekday(0), "segunda-feira");
        assert_eq!(nome_weekday(6), "domingo");
    }
}
