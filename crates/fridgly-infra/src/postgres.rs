//! Postgres implementation of [`ItemRepository`].

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use fridgly_domain::{
    Item, ItemChanges, ItemRepository, ItemStatus, NewItem, RepositoryError, Subunit,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Repository backed by a Postgres connection pool.
#[derive(Clone)]
pub struct PgItemRepository {
    pool: PgPool,
}

impl PgItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Raw row shape as stored in Postgres. `status` is a plain string column that
/// we convert into the domain [`ItemStatus`] enum on the way out.
#[derive(FromRow)]
struct ItemRow {
    id: Uuid,
    name: String,
    quantity: String,
    unit: Option<String>,
    subunit_remaining: Option<i32>,
    subunit_unit: Option<String>,
    category: Option<String>,
    expiry_date: Option<NaiveDate>,
    status: String,
    added_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ItemRow> for Item {
    type Error = RepositoryError;

    fn try_from(row: ItemRow) -> Result<Self, Self::Error> {
        let status = row
            .status
            .parse::<ItemStatus>()
            .map_err(RepositoryError::Backend)?;
        Ok(Item {
            id: row.id,
            name: row.name,
            quantity: row.quantity,
            unit: row.unit,
            subunit: Subunit::new(row.subunit_remaining, row.subunit_unit),
            category: row.category,
            expiry_date: row.expiry_date,
            status,
            added_at: row.added_at,
            updated_at: row.updated_at,
        })
    }
}

/// Columns selected everywhere, kept in one place to stay consistent.
const COLUMNS: &str = "id, name, quantity, unit, subunit_remaining, subunit_unit, \
                       category, expiry_date, status, added_at, updated_at";

/// Split a domain [`Subunit`] into the two flat columns Postgres stores.
fn subunit_columns(subunit: Option<Subunit>) -> (Option<i32>, Option<String>) {
    match subunit {
        Some(s) => (Some(s.remaining), s.unit),
        None => (None, None),
    }
}

fn backend<E: std::fmt::Display>(e: E) -> RepositoryError {
    RepositoryError::Backend(e.to_string())
}

#[async_trait]
impl ItemRepository for PgItemRepository {
    async fn list_in_fridge(&self) -> Result<Vec<Item>, RepositoryError> {
        let sql = format!(
            "SELECT {COLUMNS} FROM items \
             WHERE status = 'in_fridge' \
             ORDER BY expiry_date ASC NULLS LAST, name ASC"
        );
        let rows = sqlx::query_as::<_, ItemRow>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(backend)?;
        rows.into_iter().map(Item::try_from).collect()
    }

    async fn search_in_fridge(&self, query: &str) -> Result<Vec<Item>, RepositoryError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.list_in_fridge().await;
        }
        // Escape LIKE wildcards so the user's text is matched literally.
        let escaped = trimmed
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        let sql = format!(
            "SELECT {COLUMNS} FROM items \
             WHERE status = 'in_fridge' AND name ILIKE $1 ESCAPE '\\' \
             ORDER BY expiry_date ASC NULLS LAST, name ASC"
        );
        let rows = sqlx::query_as::<_, ItemRow>(&sql)
            .bind(pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(backend)?;
        rows.into_iter().map(Item::try_from).collect()
    }

    async fn find(&self, id: Uuid) -> Result<Option<Item>, RepositoryError> {
        let sql = format!("SELECT {COLUMNS} FROM items WHERE id = $1");
        let row = sqlx::query_as::<_, ItemRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(backend)?;
        row.map(Item::try_from).transpose()
    }

    async fn create(&self, new_item: NewItem) -> Result<Item, RepositoryError> {
        let (subunit_remaining, subunit_unit) = subunit_columns(new_item.subunit);
        let sql = format!(
            "INSERT INTO items (name, quantity, unit, subunit_remaining, subunit_unit, category, expiry_date) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING {COLUMNS}"
        );
        let row = sqlx::query_as::<_, ItemRow>(&sql)
            .bind(new_item.name)
            .bind(new_item.quantity)
            .bind(new_item.unit)
            .bind(subunit_remaining)
            .bind(subunit_unit)
            .bind(new_item.category)
            .bind(new_item.expiry_date)
            .fetch_one(&self.pool)
            .await
            .map_err(backend)?;
        Item::try_from(row)
    }

    async fn update(&self, id: Uuid, changes: ItemChanges) -> Result<Item, RepositoryError> {
        let (subunit_remaining, subunit_unit) = subunit_columns(changes.subunit);
        let sql = format!(
            "UPDATE items \
             SET name = $2, quantity = $3, unit = $4, subunit_remaining = $5, subunit_unit = $6, \
                 category = $7, expiry_date = $8, updated_at = now() \
             WHERE id = $1 AND status = 'in_fridge' RETURNING {COLUMNS}"
        );
        let row = sqlx::query_as::<_, ItemRow>(&sql)
            .bind(id)
            .bind(changes.name)
            .bind(changes.quantity)
            .bind(changes.unit)
            .bind(subunit_remaining)
            .bind(subunit_unit)
            .bind(changes.category)
            .bind(changes.expiry_date)
            .fetch_optional(&self.pool)
            .await
            .map_err(backend)?
            .ok_or(RepositoryError::NotFound)?;
        Item::try_from(row)
    }

    async fn set_status(&self, id: Uuid, status: ItemStatus) -> Result<(), RepositoryError> {
        let result = sqlx::query("UPDATE items SET status = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(status.as_str())
            .execute(&self.pool)
            .await
            .map_err(backend)?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(backend)?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
