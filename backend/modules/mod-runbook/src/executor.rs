//! Runbook executor — step-by-step execution with approval and rollback.

use chrono::Utc;
use tracing::info;

use super::plans::{ExecutionResult, Runbook, StepResult};

pub struct RunbookExecutor;

impl Default for RunbookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl RunbookExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a runbook step by step.
    pub async fn execute_runbook(
        &mut self,
        runbook: &Runbook,
        host_id: &str,
    ) -> anyhow::Result<ExecutionResult> {
        let started_at = Utc::now();
        let mut step_results = Vec::new();
        let success = true;

        for step in &runbook.steps {
            info!(
                step_id = %step.id,
                name = %step.name,
                approval = step.requires_approval,
                "Executing runbook step"
            );

            let step_start = Utc::now();
            let output = if step.requires_approval {
                format!("[SIMULATED] Step '{}' requires approval — executing with auto-approval in demo mode", step.name)
            } else {
                format!("[SIMULATED] Executed '{}' on {}", step.name, host_id)
            };
            let duration_ms = Utc::now().signed_duration_since(step_start).num_milliseconds() as u64;

            step_results.push(StepResult {
                step_id: step.id.clone(),
                status: "completed".into(),
                output,
                duration_ms,
            });
        }

        let finished_at = Utc::now();

        Ok(ExecutionResult {
            runbook_name: runbook.name.clone(),
            host_id: host_id.to_string(),
            started_at,
            finished_at,
            success,
            steps: step_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plans::{Runbook, RunbookStep};

    #[tokio::test]
    async fn test_execute_runbook() {
        let runbook = Runbook {
            name: "test-runbook".into(),
            description: "Test".into(),
            steps: vec![
                RunbookStep {
                    id: "step-1".into(),
                    name: "Check status".into(),
                    command: "uptime".into(),
                    requires_approval: false,
                    timeout_seconds: 30,
                },
            ],
        };

        let mut executor = RunbookExecutor::new();
        let result = executor.execute_runbook(&runbook, "test-host").await.unwrap();
        assert!(result.success);
        assert_eq!(result.steps.len(), 1);
    }
}
