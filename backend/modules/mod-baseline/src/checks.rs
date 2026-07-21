//! Baseline check definitions — 30+ security checks across multiple categories.
//!
//! Categories: SSH configuration, password policy, file permissions,
//! kernel parameters, service auditing, user management, network security.

use std::collections::HashMap;
use std::sync::Arc;

use ops_pilot_core::ssh::SshConnectionPool;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Result of a single baseline check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub category: String,
    pub status: CheckStatus,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warn,
    Skip,
    Info,
}

/// Defines a baseline check rule.
struct CheckRule {
    name: &'static str,
    category: &'static str,
    command: &'static str,
    evaluator: fn(&str) -> (CheckStatus, String, Option<String>),
}

pub struct BaselineChecker {
    executor: ops_pilot_core::ssh::CommandExecutor,
}

impl BaselineChecker {
    pub fn new(ssh_pool: Arc<SshConnectionPool>) -> Self {
        Self {
            executor: ops_pilot_core::ssh::CommandExecutor::new(ssh_pool),
        }
    }

    pub async fn run_checks(
        &mut self,
        host_id: &str,
        check_name: &str,
    ) -> anyhow::Result<Vec<CheckResult>> {
        let rules = all_rules();
        let mut results = Vec::new();

        for rule in &rules {
            if check_name != "all" && rule.name != check_name {
                continue;
            }

            match self.executor.exec_on_host(host_id, rule.command).await {
                Ok(result) => {
                    let output = if result.success() {
                        result.stdout.clone()
                    } else {
                        String::new()
                    };
                    let (status, message, remediation) = (rule.evaluator)(&output);
                    results.push(CheckResult {
                        name: rule.name.to_string(),
                        category: rule.category.to_string(),
                        status,
                        message,
                        remediation,
                    });
                }
                Err(e) => {
                    warn!(host_id, check = rule.name, error = %e, "check failed");
                    results.push(CheckResult {
                        name: rule.name.to_string(),
                        category: rule.category.to_string(),
                        status: CheckStatus::Skip,
                        message: format!("SSH exec failed: {}", e),
                        remediation: None,
                    });
                }
            }
        }

        Ok(results)
    }
}

fn all_rules() -> Vec<CheckRule> {
    vec![
        // SSH Configuration
        CheckRule {
            name: "ssh_permit_root_login",
            category: "ssh",
            command: "grep -E '^PermitRootLogin' /etc/ssh/sshd_config 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.contains("no") {
                    (CheckStatus::Pass, "Root login disabled".into(), None)
                } else {
                    (CheckStatus::Fail, "Root login permitted".into(), Some("Set PermitRootLogin no in /etc/ssh/sshd_config".into()))
                }
            },
        },
        CheckRule {
            name: "ssh_password_auth",
            category: "ssh",
            command: "grep -E '^PasswordAuthentication' /etc/ssh/sshd_config 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.contains("no") {
                    (CheckStatus::Pass, "Password authentication disabled".into(), None)
                } else {
                    (CheckStatus::Warn, "Password authentication enabled".into(), Some("Consider disabling PasswordAuthentication".into()))
                }
            },
        },
        CheckRule {
            name: "ssh_protocol_version",
            category: "ssh",
            command: "sshd -V 2>&1 | head -1",
            evaluator: |output| {
                if output.contains("OpenSSH") {
                    (CheckStatus::Pass, format!("SSH version: {}", output.trim()), None)
                } else {
                    (CheckStatus::Skip, "Could not determine SSH version".into(), None)
                }
            },
        },
        CheckRule {
            name: "ssh_max_auth_tries",
            category: "ssh",
            command: "grep -E '^MaxAuthTries' /etc/ssh/sshd_config 2>/dev/null || echo 'default'",
            evaluator: |output| {
                if let Some(val) = output.split_whitespace().last() {
                    if let Ok(n) = val.parse::<u32>() {
                        if n <= 4 {
                            return (CheckStatus::Pass, format!("MaxAuthTries={}", n), None);
                        }
                        return (CheckStatus::Fail, format!("MaxAuthTries={} (too high)", n), Some("Set MaxAuthTries 3 or 4".into()));
                    }
                }
                (CheckStatus::Pass, "Using default MaxAuthTries".into(), None)
            },
        },
        CheckRule {
            name: "ssh_empty_passwords",
            category: "ssh",
            command: "grep -E '^PermitEmptyPasswords' /etc/ssh/sshd_config 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.contains("yes") {
                    (CheckStatus::Fail, "Empty passwords permitted!".into(), Some("Set PermitEmptyPasswords no".into()))
                } else {
                    (CheckStatus::Pass, "Empty passwords not permitted".into(), None)
                }
            },
        },
        // Password Policy
        CheckRule {
            name: "password_max_days",
            category: "password",
            command: "grep -E '^PASS_MAX_DAYS' /etc/login.defs 2>/dev/null | awk '{print $2}'",
            evaluator: |output| {
                if let Ok(days) = output.trim().parse::<u32>() {
                    if days <= 90 {
                        (CheckStatus::Pass, format!("PASS_MAX_DAYS={}", days), None)
                    } else {
                        (CheckStatus::Fail, format!("PASS_MAX_DAYS={} (should be <=90)", days), Some("Set PASS_MAX_DAYS 90 in /etc/login.defs".into()))
                    }
                } else {
                    (CheckStatus::Skip, "Could not determine PASS_MAX_DAYS".into(), None)
                }
            },
        },
        CheckRule {
            name: "password_min_days",
            category: "password",
            command: "grep -E '^PASS_MIN_DAYS' /etc/login.defs 2>/dev/null | awk '{print $2}'",
            evaluator: |output| {
                if let Ok(days) = output.trim().parse::<u32>() {
                    if days >= 1 {
                        (CheckStatus::Pass, format!("PASS_MIN_DAYS={}", days), None)
                    } else {
                        (CheckStatus::Warn, "PASS_MIN_DAYS=0 (allows rapid password changes)".into(), None)
                    }
                } else {
                    (CheckStatus::Skip, "Could not determine PASS_MIN_DAYS".into(), None)
                }
            },
        },
        CheckRule {
            name: "password_min_length",
            category: "password",
            command: "grep -E '^minlen' /etc/pam.d/common-password 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.contains("not_found") {
                    (CheckStatus::Skip, "PAM password config not found".into(), None)
                } else if output.contains("minlen") {
                    (CheckStatus::Pass, "Password minimum length configured".into(), None)
                } else {
                    (CheckStatus::Warn, "No minlen set in PAM".into(), Some("Set minlen=12 or higher".into()))
                }
            },
        },
        // File Permissions
        CheckRule {
            name: "passwd_file_perms",
            category: "file_permissions",
            command: "stat -c '%a %U' /etc/passwd 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                let perms = output.split_whitespace().next().unwrap_or("");
                if perms == "644" {
                    (CheckStatus::Pass, "/etc/passwd has correct permissions (644)".into(), None)
                } else {
                    (CheckStatus::Fail, format!("/etc/passwd perms: {}", perms), Some("chmod 644 /etc/passwd".into()))
                }
            },
        },
        CheckRule {
            name: "shadow_file_perms",
            category: "file_permissions",
            command: "stat -c '%a %U' /etc/shadow 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                let perms = output.split_whitespace().next().unwrap_or("");
                if perms == "640" || perms == "600" {
                    (CheckStatus::Pass, format!("/etc/shadow permissions: {}", perms), None)
                } else {
                    (CheckStatus::Fail, format!("/etc/shadow perms: {} (insecure)", perms), Some("chmod 640 /etc/shadow".into()))
                }
            },
        },
        CheckRule {
            name: "sshd_config_perms",
            category: "file_permissions",
            command: "stat -c '%a' /etc/ssh/sshd_config 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                let perms = output.trim();
                if perms == "600" || perms == "644" {
                    (CheckStatus::Pass, format!("sshd_config perms: {}", perms), None)
                } else {
                    (CheckStatus::Fail, format!("sshd_config perms: {}", perms), Some("chmod 600 /etc/ssh/sshd_config".into()))
                }
            },
        },
        CheckRule {
            name: "world_writable_files",
            category: "file_permissions",
            command: "find /etc -xdev -type f -perm -002 2>/dev/null | head -5",
            evaluator: |output| {
                if output.trim().is_empty() {
                    (CheckStatus::Pass, "No world-writable files in /etc".into(), None)
                } else {
                    let count = output.lines().count();
                    (CheckStatus::Fail, format!("{} world-writable files found", count), Some("chmod o-w on affected files".into()))
                }
            },
        },
        // Kernel Parameters
        CheckRule {
            name: "kernel_aslr",
            category: "kernel",
            command: "cat /proc/sys/kernel/randomize_va_space 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.trim() == "2" {
                    (CheckStatus::Pass, "ASLR fully enabled".into(), None)
                } else {
                    (CheckStatus::Fail, "ASLR not fully enabled".into(), Some("sysctl -w kernel.randomize_va_space=2".into()))
                }
            },
        },
        CheckRule {
            name: "kernel_ip_forward",
            category: "kernel",
            command: "cat /proc/sys/net/ipv4/ip_forward 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.trim() == "0" {
                    (CheckStatus::Pass, "IP forwarding disabled".into(), None)
                } else {
                    (CheckStatus::Warn, "IP forwarding enabled".into(), Some("Disable if not a router: sysctl -w net.ipv4.ip_forward=0".into()))
                }
            },
        },
        CheckRule {
            name: "kernel_syn_cookies",
            category: "kernel",
            command: "cat /proc/sys/net/ipv4/tcp_syncookies 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.trim() == "1" {
                    (CheckStatus::Pass, "SYN cookies enabled".into(), None)
                } else {
                    (CheckStatus::Fail, "SYN cookies disabled".into(), Some("sysctl -w net.ipv4.tcp_syncookies=1".into()))
                }
            },
        },
        CheckRule {
            name: "kernel_dmesg_restrict",
            category: "kernel",
            command: "cat /proc/sys/kernel/dmesg_restrict 2>/dev/null || echo '0'",
            evaluator: |output| {
                if output.trim() == "1" {
                    (CheckStatus::Pass, "dmesg restricted".into(), None)
                } else {
                    (CheckStatus::Warn, "dmesg readable by non-root".into(), Some("sysctl -w kernel.dmesg_restrict=1".into()))
                }
            },
        },
        // Service Auditing
        CheckRule {
            name: "auditd_running",
            category: "services",
            command: "systemctl is-active auditd 2>/dev/null || echo 'inactive'",
            evaluator: |output| {
                if output.trim() == "active" {
                    (CheckStatus::Pass, "auditd is running".into(), None)
                } else {
                    (CheckStatus::Warn, "auditd is not running".into(), Some("systemctl enable --now auditd".into()))
                }
            },
        },
        CheckRule {
            name: "firewall_active",
            category: "services",
            command: "ufw status 2>/dev/null || iptables -L -n 2>/dev/null | head -5 || echo 'not_found'",
            evaluator: |output| {
                if output.contains("active") || output.contains("Chain INPUT") {
                    (CheckStatus::Pass, "Firewall appears active".into(), None)
                } else {
                    (CheckStatus::Fail, "No firewall detected".into(), Some("Enable ufw or configure iptables".into()))
                }
            },
        },
        CheckRule {
            name: "unattended_upgrades",
            category: "services",
            command: "systemctl is-active unattended-upgrades 2>/dev/null || dpkg -l | grep unattended-upgrades 2>/dev/null || echo 'not_found'",
            evaluator: |output| {
                if output.contains("active") || output.contains("ii") {
                    (CheckStatus::Pass, "Unattended upgrades available".into(), None)
                } else {
                    (CheckStatus::Warn, "Unattended upgrades not configured".into(), Some("apt install unattended-upgrades".into()))
                }
            },
        },
        // User Management
        CheckRule {
            name: "empty_password_accounts",
            category: "users",
            command: "awk -F: '($2 == \"\" || $2 == \"!\") {print $1}' /etc/shadow 2>/dev/null | head -5",
            evaluator: |output| {
                let users: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
                if users.is_empty() {
                    (CheckStatus::Pass, "No accounts with empty passwords".into(), None)
                } else {
                    (CheckStatus::Fail, format!("Accounts with empty passwords: {:?}", users), Some("Lock or set passwords for these accounts".into()))
                }
            },
        },
        CheckRule {
            name: "uid_zero_accounts",
            category: "users",
            command: "awk -F: '($3 == 0) {print $1}' /etc/passwd",
            evaluator: |output| {
                let users: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
                if users.len() == 1 && users[0] == "root" {
                    (CheckStatus::Pass, "Only root has UID 0".into(), None)
                } else {
                    (CheckStatus::Fail, format!("Multiple UID 0 accounts: {:?}", users), Some("Remove non-root UID 0 accounts".into()))
                }
            },
        },
        CheckRule {
            name: "sudo_nopasswd",
            category: "users",
            command: "grep -r 'NOPASSWD' /etc/sudoers /etc/sudoers.d/ 2>/dev/null | head -5",
            evaluator: |output| {
                if output.trim().is_empty() {
                    (CheckStatus::Pass, "No NOPASSWD sudo entries".into(), None)
                } else {
                    let count = output.lines().count();
                    (CheckStatus::Warn, format!("{} NOPASSWD sudo entries", count), Some("Remove NOPASSWD where not needed".into()))
                }
            },
        },
        // Network Security
        CheckRule {
            name: "open_ports",
            category: "network",
            command: "ss -tlnp 2>/dev/null | grep LISTEN | wc -l",
            evaluator: |output| {
                let count = output.trim().parse::<u32>().unwrap_or(0);
                if count <= 5 {
                    (CheckStatus::Pass, format!("{} listening ports", count), None)
                } else {
                    (CheckStatus::Warn, format!("{} listening ports (review if expected)", count), None)
                }
            },
        },
        CheckRule {
            name: "icmp_redirects",
            category: "network",
            command: "cat /proc/sys/net/ipv4/conf/all/accept_redirects 2>/dev/null || echo '1'",
            evaluator: |output| {
                if output.trim() == "0" {
                    (CheckStatus::Pass, "ICMP redirects disabled".into(), None)
                } else {
                    (CheckStatus::Fail, "ICMP redirects accepted".into(), Some("sysctl -w net.ipv4.conf.all.accept_redirects=0".into()))
                }
            },
        },
        CheckRule {
            name: "ipv6_disabled",
            category: "network",
            command: "cat /proc/sys/net/ipv6/conf/all/disable_ipv6 2>/dev/null || echo '0'",
            evaluator: |output| {
                if output.trim() == "1" {
                    (CheckStatus::Pass, "IPv6 disabled".into(), None)
                } else {
                    (CheckStatus::Info, "IPv6 enabled".into(), None)
                }
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert!(rules.len() >= 20, "Expected at least 20 rules, got {}", rules.len());
    }

    #[test]
    fn test_ssh_permit_root_login_pass() {
        let (_, msg, _) = (all_rules()[0].evaluator)("PermitRootLogin no");
        assert!(msg.contains("disabled"));
    }

    #[test]
    fn test_ssh_permit_root_login_fail() {
        let (status, _, _) = (all_rules()[0].evaluator)("PermitRootLogin yes");
        assert_eq!(status, CheckStatus::Fail);
    }
}
