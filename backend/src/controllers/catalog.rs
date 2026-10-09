use axum::routing::get;
use loco_rs::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::models::tenants::TenantModel;
use crate::models::services::ServiceModel;
use crate::models::professionals::ProfessionalModel;

#[derive(Debug, Serialize)]
pub struct CatalogResponse {
    pub tenant: TenantInfo,
    pub services: Vec<ServiceInfo>,
    pub professionals: Vec<ProfessionalInfo>,
}

#[derive(Debug, Serialize)]
pub struct TenantInfo {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price_in_cents: i64,
    pub signal_percent: i16,
}

#[derive(Debug, Serialize)]
pub struct ProfessionalInfo {
    pub id: Uuid,
    pub name: String,
}

async fn get_catalog(
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
) -> Result<Json<CatalogResponse>> {
    let tenant = TenantModel::find_by_slug(&ctx.db, &slug)
        .await?
        .ok_or_else(|| Error::not_found("tenant não encontrado"))?;

    let services = ServiceModel::find_by_tenant(&ctx.db, tenant.id).await?;
    let professionals = ProfessionalModel::find_by_tenant(&ctx.db, tenant.id).await?;

    Ok(Json(CatalogResponse {
        tenant: TenantInfo {
            id: tenant.id,
            name: tenant.name,
            slug: tenant.slug,
        },
        services: services
            .into_iter()
            .map(|s| ServiceInfo {
                id: s.id,
                name: s.name,
                description: s.description,
                duration_minutes: s.duration_minutes,
                price_in_cents: s.price_in_cents,
                signal_percent: s.signal_percent,
            })
            .collect(),
        professionals: professionals
            .into_iter()
            .map(|p| ProfessionalInfo {
                id: p.id,
                name: p.name,
            })
            .collect(),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("public")
        .add("/{slug}/catalog", get(get_catalog))
}