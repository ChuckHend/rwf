use crate::model::{FromRow, Model, ToValue, Value};
use time::OffsetDateTime;

use std::path::PathBuf;

use super::Direction;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Migration {
    id: Option<i64>,
    pub version: i64,
    pub name: String,
    pub applied_at: Option<OffsetDateTime>,
}

impl FromRow for Migration {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            version: row.get("version"),
            name: row.get("name"),
            applied_at: row.get("applied_at"),
        }
    }
}

impl Model for Migration {
    fn primary_key() -> &'static str {
        "id"
    }

    fn table_name() -> &'static str {
        "rum_migrations"
    }

    fn foreign_key() -> &'static str {
        "rum_migration_id"
    }

    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.version.to_value(),
            self.name.to_value(),
            self.applied_at.to_value(),
        ]
    }

    fn column_names() -> &'static [&'static str] {
        &["version", "name", "applied_at"]
    }
}

impl Migration {
    pub(crate) fn path(&self, direction: Direction) -> PathBuf {
        PathBuf::from(format!(
            "{}_{}.{}.sql",
            self.version,
            self.name,
            match direction {
                Direction::Up => "up",
                Direction::Down => "down",
            }
        ))
    }
}