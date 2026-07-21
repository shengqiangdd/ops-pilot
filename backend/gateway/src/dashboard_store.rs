//! 仪表盘布局存储 —— CRUD 管理仪表盘布局配置。

use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardLayout {
    pub id: String,
    pub name: String,
    pub layout_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveLayoutRequest {
    pub name: String,
    pub layout_json: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLayoutRequest {
    pub name: Option<String>,
    pub layout_json: Option<String>,
}

/// 保存仪表盘布局（新建或更新）。
pub async fn save_layout(
    pool: &SqlitePool,
    id: Option<&str>,
    req: &SaveLayoutRequest,
) -> Result<DashboardLayout, String> {
    let layout_id = id.map(String::from).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    sqlx::query(
        "INSERT INTO dashboard_layouts (id, name, layout_json, created_at, updated_at) \
         VALUES (?, ?, ?, datetime('now'), datetime('now')) \
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, layout_json = excluded.layout_json, \
         updated_at = datetime('now')",
    )
    .bind(&layout_id)
    .bind(&req.name)
    .bind(&req.layout_json)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    load_layout(pool, &layout_id).await
}

/// 加载单个仪表盘布局。
pub async fn load_layout(pool: &SqlitePool, id: &str) -> Result<DashboardLayout, String> {
    sqlx::query_as::<_, DashboardLayout>(
        "SELECT id, name, layout_json, created_at, updated_at FROM dashboard_layouts WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("layout not found: {id}"))
}

/// 列出所有仪表盘布局。
pub async fn list_layouts(pool: &SqlitePool) -> Result<Vec<DashboardLayout>, String> {
    sqlx::query_as::<_, DashboardLayout>(
        "SELECT id, name, layout_json, created_at, updated_at FROM dashboard_layouts ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

/// 删除仪表盘布局。
pub async fn delete_layout(pool: &SqlitePool, id: &str) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM dashboard_layouts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}
