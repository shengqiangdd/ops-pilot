//! Multi-Cluster 管理 —— 集群注册、状态查询、指标汇总。

use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cluster {
    pub id: String,
    pub name: String,
    pub api_server: String,
    pub token: Option<String>,
    pub status: String,
    pub metrics_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterClusterRequest {
    pub name: String,
    pub api_server: String,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClusterRequest {
    pub name: Option<String>,
    pub api_server: Option<String>,
    pub token: Option<String>,
    pub status: Option<String>,
    pub metrics_json: Option<String>,
}

pub struct ClusterManager {
    pool: SqlitePool,
}

impl ClusterManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 注册新集群。
    pub async fn register(&self, req: &RegisterClusterRequest) -> Result<Cluster, String> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO clusters (id, name, api_server, token, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 'unknown', datetime('now'), datetime('now'))",
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.api_server)
        .bind(&req.token)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.get(&id).await
    }

    /// 获取单个集群。
    pub async fn get(&self, id: &str) -> Result<Cluster, String> {
        sqlx::query_as::<_, Cluster>(
            "SELECT id, name, api_server, token, status, metrics_json, created_at, updated_at \
             FROM clusters WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("cluster not found: {id}"))
    }

    /// 列出所有集群。
    pub async fn list(&self) -> Result<Vec<Cluster>, String> {
        sqlx::query_as::<_, Cluster>(
            "SELECT id, name, api_server, token, status, metrics_json, created_at, updated_at \
             FROM clusters ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    /// 更新集群信息。
    pub async fn update(&self, id: &str, req: &UpdateClusterRequest) -> Result<Cluster, String> {
        if let Some(v) = &req.name {
            sqlx::query("UPDATE clusters SET name = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(v).bind(id).execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(v) = &req.api_server {
            sqlx::query("UPDATE clusters SET api_server = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(v).bind(id).execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(v) = &req.token {
            sqlx::query("UPDATE clusters SET token = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(v).bind(id).execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(v) = &req.status {
            sqlx::query("UPDATE clusters SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(v).bind(id).execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(v) = &req.metrics_json {
            sqlx::query("UPDATE clusters SET metrics_json = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(v).bind(id).execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        self.get(id).await
    }

    /// 删除集群。
    pub async fn delete(&self, id: &str) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM clusters WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    /// 获取集群状态（模拟健康检查）。
    pub async fn status(&self, id: &str) -> Result<ClusterStatus, String> {
        let cluster = self.get(id).await?;

        // 模拟状态检查（实际应调用 k8s API）
        let status = if cluster.status == "online" {
            ClusterStatus {
                id: cluster.id,
                name: cluster.name,
                status: "healthy".into(),
                api_reachable: true,
                node_count: 3,
                version: "1.28.4".into(),
            }
        } else {
            ClusterStatus {
                id: cluster.id,
                name: cluster.name,
                status: cluster.status,
                api_reachable: false,
                node_count: 0,
                version: "unknown".into(),
            }
        };

        Ok(status)
    }
}

#[derive(Debug, Serialize)]
pub struct ClusterStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub api_reachable: bool,
    pub node_count: i32,
    pub version: String,
}
