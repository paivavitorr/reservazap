use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::_entities::users::{self, Entity as User};

pub struct UserModel;

impl UserModel {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<users::Model>, sea_orm::DbErr> {
        User::find_by_id(id).one(db).await
    }

    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<users::Model>, sea_orm::DbErr> {
        User::find()
            .filter(users::Column::Email.eq(email.to_lowercase()))
            .one(db)
            .await
    }

    pub async fn criar<C: sea_orm::ConnectionTrait>(
        db: &C,
        tenant_id: Uuid,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<users::Model, sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        users::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            name: Set(name.to_string()),
            email: Set(email.to_lowercase()),
            password_hash: Set(password_hash.to_string()),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
    }
}
