//! AI 告警诊断 —— 根据告警历史、规则分析和知识库匹配生成诊断建议。

use sqlx::SqlitePool;
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AlertHistoryEntry {
    pub id: String,
    pub message: String,
    pub severity: String,
    pub resource: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnosisResult {
    pub alert: AlertHistoryEntry,
    pub rule_analysis: String,
    pub suggestions: Vec<String>,
    pub knowledge_matches: Vec<KnowledgeMatch>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KnowledgeMatch {
    pub id: String,
    pub title: String,
    pub root_cause: String,
    pub resolution: String,
}

/// 诊断一条告警：查询历史 → 规则分析 → 知识库匹配 → 返回建议。
pub async fn diagnose_alert(pool: &SqlitePool, alert_id: &str) -> Result<DiagnosisResult, String> {
    // 1. 查询告警历史
    let alert = sqlx::query_as::<_, AlertHistoryEntry>(
        "SELECT id, message, severity, resource, created_at FROM alert_history WHERE id = ?",
    )
    .bind(alert_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("alert not found: {alert_id}"))?;

    // 2. 规则分析
    let rule_analysis = generate_rule_analysis(&alert);

    // 3. 知识库匹配
    let knowledge_matches = match_knowledge(pool, &alert.message, &alert.resource).await?;

    // 4. 生成建议
    let suggestions = generate_suggestions(&alert, &knowledge_matches);

    Ok(DiagnosisResult {
        alert,
        rule_analysis,
        suggestions,
        knowledge_matches,
    })
}

/// 根据告警类型生成规则分析文本。
fn generate_rule_analysis(alert: &AlertHistoryEntry) -> String {
    let msg_lower = alert.message.to_lowercase();
    let resource = &alert.resource;

    if msg_lower.contains("cpu") {
        format!(
            "CPU 告警分析: 主机 {resource} 触发了 CPU 告警。可能原因包括: \
             进程占用过高、死循环、资源不足。建议检查 top/htop 中的高 CPU 进程。"
        )
    } else if msg_lower.contains("memory") || msg_lower.contains("mem") {
        format!(
            "内存告警分析: 主机 {resource} 内存使用超阈值。可能原因: \
             内存泄漏、缓存未释放、OOM 风险。建议检查 /proc/meminfo 和进程内存占用。"
        )
    } else if msg_lower.contains("disk") {
        format!(
            "磁盘告警分析: 主机 {resource} 磁盘空间不足。可能原因: \
             日志膨胀、临时文件堆积、大文件写入。建议 du -sh /* 排查大目录。"
        )
    } else if msg_lower.contains("ssh") || msg_lower.contains("connection") {
        format!(
            "连接告警分析: 主机 {resource} SSH 连接异常。可能原因: \
             网络中断、sshd 崩溃、防火墙规则变更、密钥过期。"
        )
    } else if msg_lower.contains("ssl") || msg_lower.contains("certificate") {
        format!(
            "证书告警分析: 主机 {resource} SSL 证书即将过期。 \
             建议通过 certbot 自动续期或手动更新证书并重启相关服务。"
        )
    } else if msg_lower.contains("service") || msg_lower.contains("health") {
        format!(
            "服务告警分析: {resource} 服务健康检查异常。 \
             可能原因: 进程崩溃、端口未监听、依赖服务不可用。建议检查 systemctl status 和端口监听。"
        )
    } else {
        format!(
            "告警分析: {resource} 触发了 [{severity}] 级别告警: \"{message}\"。 \
             建议查看相关日志并结合历史告警进行关联分析。",
            severity = alert.severity,
            message = alert.message,
        )
    }
}

/// 从知识库匹配相关条目。
async fn match_knowledge(
    pool: &SqlitePool,
    message: &str,
    resource: &str,
) -> Result<Vec<KnowledgeMatch>, String> {
    let keywords = extract_keywords(message);
    let resource_keyword = resource.to_lowercase();

    let mut matches = Vec::new();

    // 按关键词搜索知识库
    for keyword in &keywords {
        let pattern = format!("%{keyword}%");
        let rows = sqlx::query_as::<_, KnowledgeMatch>(
            "SELECT id, title, root_cause, resolution FROM knowledge_entries \
             WHERE title LIKE ? OR root_cause LIKE ? OR resolution LIKE ? \
             LIMIT 3",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for row in rows {
            if !matches.iter().any(|m: &KnowledgeMatch| m.id == row.id) {
                matches.push(row);
            }
        }
    }

    // 补充按资源名搜索
    if !resource_keyword.is_empty() {
        let pattern = format!("%{resource_keyword}%");
        let rows = sqlx::query_as::<_, KnowledgeMatch>(
            "SELECT id, title, root_cause, resolution FROM knowledge_entries \
             WHERE title LIKE ? OR root_cause LIKE ? \
             LIMIT 3",
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for row in rows {
            if !matches.iter().any(|m: &KnowledgeMatch| m.id == row.id) {
                matches.push(row);
            }
        }
    }

    Ok(matches)
}

/// 从告警消息中提取关键词。
fn extract_keywords(message: &str) -> Vec<String> {
    let stop_words: Vec<&str> = vec![
        "on", "the", "a", "an", "is", "was", "has", "had", "and", "or",
        "for", "to", "of", "in", "at", "by", "with", "from", "as",
    ];
    message
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// 根据告警信息和知识库匹配生成建议列表。
fn generate_suggestions(
    alert: &AlertHistoryEntry,
    knowledge: &[KnowledgeMatch],
) -> Vec<String> {
    let mut suggestions = Vec::new();

    // 根据严重级别添加通用建议
    match alert.severity.as_str() {
        "critical" => {
            suggestions.push("优先级: 立即处理".into());
            suggestions.push("建议通知值班人员并启动应急响应流程".into());
        }
        "warning" => {
            suggestions.push("优先级: 尽快处理".into());
            suggestions.push("建议在 30 分钟内排查".into());
        }
        _ => {
            suggestions.push("优先级: 常规处理".into());
            suggestions.push("可在下次维护窗口处理".into());
        }
    }

    // 从知识库匹配中提取具体建议
    for k in knowledge.iter().take(3) {
        suggestions.push(format!("知识库建议 [{}]: {}", k.title, k.resolution));
    }

    // 通用排查步骤
    let msg_lower = alert.message.to_lowercase();
    if msg_lower.contains("cpu") || msg_lower.contains("memory") {
        suggestions.push("排查步骤: top -c → 定位高占用进程 → 分析是否为已知进程".into());
    }
    if msg_lower.contains("disk") {
        suggestions.push("排查步骤: df -h → du -sh /* → 定位大目录 → 清理或扩容".into());
    }
    if msg_lower.contains("ssh") || msg_lower.contains("connection") {
        suggestions.push("排查步骤: ping → telnet <ip> 22 → systemctl status sshd → 检查防火墙".into());
    }

    suggestions
}
