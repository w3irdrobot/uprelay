use crate::models::{FeeSchedule, Limitation, Relay};
use ::time::OffsetDateTime;
use anyhow::{anyhow, Result};
use log::info;
use serde_json::json;
use sqlx::{migrate::Migrator, types::Json, Connection, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use time::PrimitiveDateTime;
use url::Url;

static MIGRATOR: Migrator = sqlx::migrate!();

pub type DbPool = Arc<Database>;

pub async fn new_db_pool(url: &str) -> Result<DbPool> {
    let pool = PgPool::connect(url).await?;
    let db = Arc::new(Database::new(pool).await?);

    Ok(db)
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(pool: PgPool) -> Result<Self> {
        pool.acquire().await?.ping().await?;
        MIGRATOR.run(&pool).await?;

        info!("connected to database");
        Ok(Self { pool })
    }

    pub async fn get_relay(&self, url: &Url) -> Result<Relay> {
        sqlx::query_as!(
            Relay,
            r#"
            SELECT
                url,
                name,
                description,
                pubkey,
                contact,
                supported_nips as "supported_nips: Json<Vec<i32>>",
                software,
                version,
                limitation as "limitation: Json<Limitation>",
                retention,
                relay_countries as "relay_countries: Json<Vec<String>>",
                language_tags as "language_tags: Json<Vec<String>>",
                tags as "tags: Json<Vec<String>>",
                posting_policy,
                payments_url,
                fees as "fees: Json<HashMap<String, Vec<FeeSchedule>>>",
                icon,
                created_at,
                updated_at,
                seen
            FROM relays
            WHERE url = $1
            "#,
            url.to_string()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!(e))
    }

    pub async fn save_relay(&self, relay: &Relay) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO relays
            (url, name, description, pubkey, contact, supported_nips, software, version, limitation,
                retention, relay_countries, language_tags, tags, posting_policy, payments_url, fees, icon)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        "#,
            relay.url,
            relay.name,
            relay.description,
            relay.pubkey,
            relay.contact,
            relay.supported_nips.as_ref().map(|v| json!(v)),
            relay.software,
            relay.version,
            relay.limitation.as_ref().map(|v| json!(v)),
            relay.retention,
            relay.relay_countries.as_ref().map(|v| json!(v)),
            relay.language_tags.as_ref().map(|v| json!(v)),
            relay.tags.as_ref().map(|v| json!(v)),
            relay.posting_policy,
            relay.payments_url,
            relay.fees.as_ref().map(|v| json!(v)),
            relay.icon,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| anyhow!(e))
    }

    pub async fn update_relay(&self, relay: &Relay) -> Result<()> {
        let now = OffsetDateTime::now_utc();
        sqlx::query!(
            r#"
            UPDATE relays
            SET (name, description, pubkey, contact, supported_nips, software, version, limitation, retention,
                relay_countries, language_tags, tags, posting_policy, payments_url, fees, icon, updated_at, seen)
            = ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        "#,
            relay.name,
            relay.description,
            relay.pubkey,
            relay.contact,
            relay.supported_nips.as_ref().map(|v| json!(v)),
            relay.software,
            relay.version,
            relay.limitation.as_ref().map(|v| json!(v)),
            relay.retention,
            relay.relay_countries.as_ref().map(|v| json!(v)),
            relay.language_tags.as_ref().map(|v| json!(v)),
            relay.tags.as_ref().map(|v| json!(v)),
            relay.posting_policy,
            relay.payments_url,
            relay.fees.as_ref().map(|v| json!(v)),
            relay.icon,
            PrimitiveDateTime::new(now.date(), now.time()),
            true
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| anyhow!(e))
    }
}
