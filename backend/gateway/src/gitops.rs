//! GitOps 配置同步 —— 通过 git 操作实现配置文件的版本管理和同步。

use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct GitOps {
    pub repo_path: PathBuf,
    pub branch: String,
    pub remote: String,
}

#[derive(Debug, Serialize)]
pub struct GitStatus {
    pub repo_path: String,
    pub branch: String,
    pub dirty: bool,
    pub ahead: i32,
    pub behind: i32,
    pub last_commit: String,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub commit_hash: Option<String>,
}

impl GitOps {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            branch: "main".into(),
            remote: "origin".into(),
        }
    }

    /// 初始化 git 仓库（如果 .git 不存在）。
    pub fn init(&self) -> Result<(), String> {
        let git_dir = self.repo_path.join(".git");
        if !git_dir.exists() {
            let output = Command::new("git")
                .arg("init")
                .current_dir(&self.repo_path)
                .output()
                .map_err(|e| format!("git init failed: {e}"))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("git init failed: {stderr}"));
            }
        }
        Ok(())
    }

    /// 执行 git add -A && git commit && git push。
    pub fn sync(&self, message: &str) -> Result<SyncResult, String> {
        self.init()?;

        // git add -A
        let add_output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git add failed: {e}"))?;

        if !add_output.status.success() {
            let stderr = String::from_utf8_lossy(&add_output.stderr);
            return Err(format!("git add failed: {stderr}"));
        }

        // git commit
        let commit_output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git commit failed: {e}"))?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            // "nothing to commit" 不算错误
            if stderr.contains("nothing to commit") {
                return Ok(SyncResult {
                    success: true,
                    message: "No changes to commit".into(),
                    commit_hash: None,
                });
            }
            return Err(format!("git commit failed: {stderr}"));
        }

        // 获取 commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git rev-parse failed: {e}"))?;

        let commit_hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();

        // git push（允许失败，远程可能未配置）
        let push_output = Command::new("git")
            .args(["push", &self.remote, &self.branch])
            .current_dir(&self.repo_path)
            .output();

        let push_msg = match push_output {
            Ok(out) if out.status.success() => "Pushed successfully".into(),
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                format!("Commit OK, push skipped: {stderr}")
            }
            Err(e) => format!("Commit OK, push failed: {e}"),
        };

        Ok(SyncResult {
            success: true,
            message: push_msg,
            commit_hash: Some(commit_hash),
        })
    }

    /// 获取 git 状态信息。
    pub fn status(&self) -> Result<GitStatus, String> {
        self.init()?;

        // 获取当前分支
        let branch_output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git branch failed: {e}"))?;

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        // 检查是否有未提交的更改
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git status failed: {e}"))?;

        let dirty = !String::from_utf8_lossy(&status_output.stdout)
            .trim()
            .is_empty();

        // 获取 ahead/behind
        let (ahead, behind) = self.get_ahead_behind().unwrap_or((0, 0));

        // 获取最后一次 commit 信息
        let log_output = Command::new("git")
            .args(["log", "-1", "--format=%h %s"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git log failed: {e}"))?;

        let last_commit = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(GitStatus {
            repo_path: self.repo_path.to_string_lossy().to_string(),
            branch,
            dirty,
            ahead,
            behind,
            last_commit,
        })
    }

    fn get_ahead_behind(&self) -> Result<(i32, i32), String> {
        let output = Command::new("git")
            .args([
                "rev-list",
                "--left-right",
                "--count",
                &format!("HEAD...{}/{}", self.remote, self.branch),
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
        if parts.len() == 2 {
            let ahead = parts[0].parse().unwrap_or(0);
            let behind = parts[1].parse().unwrap_or(0);
            Ok((ahead, behind))
        } else {
            Ok((0, 0))
        }
    }
}
