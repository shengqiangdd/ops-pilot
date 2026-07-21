//! Backup and restore for OpsPilot configuration.
//!
//! Supports exporting all configuration tables as a JSON file and
//! re-importing them for migration or disaster recovery.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Column, Row, SqlitePool, TypeInfo};
use std::collections::HashMap;

/// Full system backup — versioned archive of all config tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBackup {
    pub version: String,
    pub created_at: String,
    pub tables: HashMap<String, Vec<HashMap<String, Value>>>,
}

/// Export all configuration tables.
pub async fn export_backup(pool: &SqlitePool) -> Result<SystemBackup> {
    let tables = [
        "config",
        "notification_channels",
        "alert_rules",
        "runbook",
        "schedules",
        "slo_definitions",
        "soar_playbooks",
        "module_configs",
        "roles",
        "users",
    ];

    let mut data = HashMap::new();
    for table in &tables {
        let sql = format!("SELECT * FROM \"{}\"", table);
        // Using raw_sql for dynamic table names — safe as table list is hardcoded
        if let Ok(rows) = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .fetch_all(pool)
            .await
        {
            let mut records = Vec::new();
            for row in &rows {
                let mut map = HashMap::new();
                for (i, col) in row.columns().iter().enumerate() {
                    let name = col.name().to_string();
                    let val: Value = match col.type_info().name() {
                        "TEXT" | "text" => row
                            .try_get::<String, _>(i)
                            .map(Value::String)
                            .unwrap_or(Value::Null),
                        "INTEGER" | "integer" => row
                            .try_get::<i64, _>(i)
                            .map(|v| Value::Number(v.into()))
                            .unwrap_or(Value::Null),
                        "REAL" | "real" => row
                            .try_get::<f64, _>(i).map(|v| serde_json::Number::from_f64(v)
                                        .map(Value::Number)
                                        .unwrap_or(Value::Null))
                            .unwrap_or(Value::Null),
                        _ => Value::Null,
                    };
                    map.insert(name, val);
                }
                records.push(map);
            }
            data.insert(table.to_string(), records);
        }
    }
    Ok(SystemBackup {
        version: "2.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        tables: data,
    })
}

/// Import a system backup.
///
/// Clears existing data for each table then re-inserts from the backup.
/// Runs inside a transaction for atomicity.
pub async fn import_backup(pool: &SqlitePool, backup: &SystemBackup) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let mut tx = pool.begin().await?;

    for (table, rows) in &backup.tables {
        if rows.is_empty() {
            continue;
        }

        // Clear existing data
        let clear_sql = format!("DELETE FROM \"{}\"", table);
        sqlx::raw_sql(sqlx::AssertSqlSafe(clear_sql.as_str()))
            .execute(&mut *tx)
            .await
            .ok();

        // Insert each row by building SQL with escaped values
        for row in rows {
            let cols: Vec<&String> = row.keys().collect();
            let col_str = cols
                .iter()
                .map(|k| format!("\"{}\"", k))
                .collect::<Vec<_>>()
                .join(", ");

            let val_str: Vec<String> = cols
                .iter()
                .map(|k| match row.get(*k) {
                    Some(Value::String(s)) => format!("'{}'", s.replace('\'', "''")),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Null) => "NULL".to_string(),
                    Some(Value::Bool(b)) => {
                        if *b { "1".to_string() } else { "0".to_string() }
                    }
                    _ => "NULL".to_string(),
                })
                .collect();
            let vals = val_str.join(", ");

            let full_sql =
                format!("INSERT INTO \"{}\" ({}) VALUES ({})", table, col_str, vals);
            if let Err(e) = sqlx::raw_sql(sqlx::AssertSqlSafe(full_sql.as_str()))
                .execute(&mut *tx)
                .await
            {
                results.push(format!("{}: insert error: {}", table, e));
            }
        }
        results.push(format!("{}: {} rows restored", table, rows.len()));
    }

    tx.commit().await?;
    Ok(results)
}
