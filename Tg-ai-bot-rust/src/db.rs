use sqlx::PgPool;
use anyhow::Result;

pub async fn is_admin_or_authorized(pool: &PgPool, tg_id: i64, group_id: i64) -> Result<bool> {
    let row = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(SELECT 1 FROM admins WHERE telegram_id = $1)
        "#,
        tg_id
    )
    .fetch_one(pool)
    .await?;
    if row {
        return Ok(true);
    }
    let row2 = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(SELECT 1 FROM authorized_users WHERE telegram_id = $1 AND group_id = $2)
        "#,
        tg_id,
        group_id
    )
    .fetch_one(pool)
    .await?;
    Ok(row2)
}