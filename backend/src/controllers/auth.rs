use axum::routing::{get, post};
use chrono::{NaiveTime, Utc};
use loco_rs::controller::extractor::auth;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::_entities::{tenants, working_hours};
use crate::models::tenants::TenantModel;
use crate::models::users::UserModel;

#[derive(Debug, Deserialize)]
pub struct RegistroPayload {
    pub tenant_name: String,
    pub slug: String,
    pub phone: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UsuarioInfo,
}

#[derive(Debug, Serialize)]
pub struct UsuarioInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub tenant_id: Uuid,
}

/// Cria uma conta de estabelecimento: tenant + usuário dono + expediente padrão.
async fn registro(
    State(ctx): State<AppContext>,
    Json(payload): Json<RegistroPayload>,
) -> Result<Json<AuthResponse>> {
    let nome = payload.tenant_name.trim();
    if nome.is_empty() || nome.len() > 120 {
        return Err(Error::BadRequest("nome do estabelecimento inválido".into()));
    }

    let slug = payload.slug.trim().to_lowercase();
    if !slug_valido(&slug) {
        return Err(Error::BadRequest(
            "slug inválido (use letras minúsculas, números e hífen)".into(),
        ));
    }

    let nome_usuario = payload.name.trim();
    if nome_usuario.len() < 2 || nome_usuario.len() > 120 {
        return Err(Error::BadRequest("nome inválido".into()));
    }

    let email = payload.email.trim().to_lowercase();
    if !email.contains('@') || email.len() > 160 {
        return Err(Error::BadRequest("e-mail inválido".into()));
    }

    if payload.password.len() < 8 {
        return Err(Error::BadRequest(
            "a senha precisa de ao menos 8 caracteres".into(),
        ));
    }

    let telefone: String = payload
        .phone
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if telefone.len() < 10 || telefone.len() > 13 {
        return Err(Error::BadRequest("telefone inválido".into()));
    }

    if TenantModel::find_by_slug(&ctx.db, &slug).await?.is_some() {
        return Err(Error::BadRequest("este link já está em uso".into()));
    }
    if UserModel::find_by_email(&ctx.db, &email).await?.is_some() {
        return Err(Error::BadRequest(
            "já existe uma conta com este e-mail".into(),
        ));
    }

    let hash = loco_rs::hash::hash_password(&payload.password)
        .map_err(|e| Error::string(&e.to_string()))?;

    use sea_orm::TransactionTrait;

    let txn = ctx
        .db
        .begin()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    let now = Utc::now().fixed_offset();
    let tenant_id = Uuid::new_v4();
    tenants::ActiveModel {
        id: Set(tenant_id),
        slug: Set(slug),
        name: Set(nome.to_string()),
        phone: Set(telefone),
        timezone: Set("America/Sao_Paulo".to_string()),
        pix_key: Set(None),
        pix_key_type: Set(None),
        booking_hold_minutes: Set(10),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&txn)
    .await?;

    // expediente padrão: seg–sex 09h–18h (com pausa de almoço) e sáb 09h–13h
    let t = |h, m| NaiveTime::from_hms_opt(h, m, 0).ok_or_else(|| Error::string("horário"));
    for (weekday, faixas) in [
        (0i16, vec![(9, 0, 12, 0), (13, 0, 18, 0)]),
        (1, vec![(9, 0, 12, 0), (13, 0, 18, 0)]),
        (2, vec![(9, 0, 12, 0), (13, 0, 18, 0)]),
        (3, vec![(9, 0, 12, 0), (13, 0, 18, 0)]),
        (4, vec![(9, 0, 12, 0), (13, 0, 18, 0)]),
        (5, vec![(9, 0, 13, 0)]),
    ] {
        for (sh, sm, eh, em) in faixas {
            working_hours::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                professional_id: Set(None),
                weekday: Set(weekday),
                start_time: Set(t(sh, sm)?),
                end_time: Set(t(eh, em)?),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(&txn)
            .await?;
        }
    }

    let user = UserModel::criar(&txn, tenant_id, nome_usuario, &email, &hash).await?;
    txn.commit()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let token = gerar_token(&ctx, user.id)?;

    Ok(Json(AuthResponse {
        token,
        user: UsuarioInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            tenant_id: user.tenant_id,
        },
    }))
}

async fn login(
    State(ctx): State<AppContext>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>> {
    let email = payload.email.trim().to_lowercase();
    let user = UserModel::find_by_email(&ctx.db, &email).await?;

    // hash real de uma senha inexistente: equaliza o tempo de resposta quando o
    // e-mail não está cadastrado (evita enumeração de contas por timing)
    let hash = user
        .as_ref()
        .map(|u| u.password_hash.as_str())
        .unwrap_or_else(|| hash_fantasma());
    let senha_ok = loco_rs::hash::verify_password(&payload.password, hash);

    if user.is_none() || !senha_ok {
        return Err(Error::Unauthorized("e-mail ou senha inválidos".into()));
    }
    let user = user.unwrap();
    if !user.is_active {
        return Err(Error::Unauthorized("conta desativada".into()));
    }

    let token = gerar_token(&ctx, user.id)?;
    Ok(Json(AuthResponse {
        token,
        user: UsuarioInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            tenant_id: user.tenant_id,
        },
    }))
}

async fn eu(State(ctx): State<AppContext>, auth: auth::JWT) -> Result<Json<UsuarioInfo>> {
    let user = usuario_do_token(&ctx, &auth).await?;
    Ok(Json(UsuarioInfo {
        id: user.id,
        name: user.name,
        email: user.email,
        tenant_id: user.tenant_id,
    }))
}

// ──────────────────────────── helpers compartilhados ────────────────────────────

pub fn gerar_token(ctx: &AppContext, user_id: Uuid) -> Result<String> {
    let config = ctx
        .config
        .auth
        .as_ref()
        .and_then(|a| a.jwt.as_ref())
        .ok_or_else(|| Error::string("JWT não configurado"))?;
    loco_rs::auth::jwt::JWT::new(&config.secret)
        .generate_token(config.expiration, user_id.to_string(), Default::default())
        .map_err(|e| Error::string(&e.to_string()))
}

/// Carrega o usuário autenticado pelo token JWT (validado pelo extractor).
pub async fn usuario_do_token(
    ctx: &AppContext,
    auth: &auth::JWT,
) -> Result<crate::models::_entities::users::Model> {
    let user_id = Uuid::parse_str(&auth.claims.pid)
        .map_err(|_| Error::Unauthorized("token inválido".into()))?;
    let user = UserModel::find_by_id(&ctx.db, user_id)
        .await?
        .filter(|u| u.is_active)
        .ok_or_else(|| Error::Unauthorized("usuário não encontrado".into()))?;
    Ok(user)
}

fn hash_fantasma() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        loco_rs::hash::hash_password("senha-fantasma-nao-existe").unwrap_or_default()
    })
    .as_str()
}

fn slug_valido(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 64
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("auth")
        .add("/registro", post(registro))
        .add("/login", post(login))
        .add("/eu", get(eu))
}
