//! CIS benchmark rules, vulnerability checks, and patch management definitions.

use serde::{Deserialize, Serialize};

/// Severity level for a security finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
        }
    }
}

/// A single security compliance rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub category: String,
    pub check_type: String,
    pub expected_value: String,
    pub remediation_steps: String,
}

impl SecurityRule {
    /// Whether this rule applies to the given check type filter.
    pub fn matches_check_type(&self, check_type: &str) -> bool {
        if check_type == "all" || check_type == "cis_linux" || check_type == "cis_docker" {
            return self.check_type == check_type || check_type == "all";
        }
        if check_type == "vulnerability" {
            return self.category == "vulnerability";
        }
        if check_type == "patch" {
            return self.category == "patch";
        }
        self.check_type == check_type
    }

    /// Simulate a check result for testing. In production, real host inspection
    /// would replace this. Some rules deterministically pass, others fail, to
    /// provide a realistic distribution for tool consumers.
    pub fn simulate_result(&self) -> (super::engine::CheckStatus, String, String) {
        use super::engine::CheckStatus;

        // Rules with "no" or "disabled" expected values mostly pass
        // to simulate a reasonably hardened host, with a few deliberate fails.
        let pass = matches!(
            self.id.as_str(),
            "CIS-1.1"
                | "CIS-1.2"
                | "CIS-1.3"
                | "CIS-5.1"
                | "SSH-1"
                | "SSH-2"
                | "SSH-3"
                | "DOCKER-1"
                | "DOCKER-3"
                | "VULN-1"
                | "PATCH-1"
        );

        if pass {
            (
                CheckStatus::Pass,
                self.expected_value.clone(),
                self.expected_value.clone(),
            )
        } else {
            (
                CheckStatus::Fail,
                format!("non-compliant: {}", self.id),
                self.expected_value.clone(),
            )
        }
    }
}

/// Build the built-in CIS benchmark and security rule set.
pub fn builtin_rules() -> Vec<SecurityRule> {
    vec![
        // ── Filesystem (CIS 1.x) ──────────────────────────────────────────
        SecurityRule {
            id: "CIS-1.1".into(),
            name: "Separate partition for /tmp".into(),
            description: "A separate partition should exist for /tmp to prevent denial-of-service attacks that fill the root partition.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "/tmp on separate partition".into(),
            remediation_steps: "Create a separate partition or logical volume for /tmp and mount it with appropriate options.".into(),
        },
        SecurityRule {
            id: "CIS-1.2".into(),
            name: "Set nodev option for /tmp".into(),
            description: "The nodev option should be set on the /tmp partition to prevent device files from being created.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "nodev".into(),
            remediation_steps: "Add 'nodev' option to /tmp in /etc/fstab and remount: mount -o remount,nodev /tmp".into(),
        },
        SecurityRule {
            id: "CIS-1.3".into(),
            name: "Set nosuid option for /tmp".into(),
            description: "The nosuid option should be set on /tmp to prevent SUID/SGID bits from taking effect.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "nosuid".into(),
            remediation_steps: "Add 'nosuid' option to /tmp in /etc/fstab and remount: mount -o remount,nosuid /tmp".into(),
        },
        SecurityRule {
            id: "CIS-1.4".into(),
            name: "Set noexec option for /tmp".into(),
            description: "The noexec option should be set on /tmp to prevent direct execution of binaries.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "noexec".into(),
            remediation_steps: "Add 'noexec' option to /tmp in /etc/fstab and remount: mount -o remount,noexec /tmp".into(),
        },
        SecurityRule {
            id: "CIS-1.5".into(),
            name: "Set nodev option for /var/tmp".into(),
            description: "The nodev option should be set on /var/tmp to prevent device files from being created.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "nodev".into(),
            remediation_steps: "Add 'nodev' option to /var/tmp in /etc/fstab and remount.".into(),
        },
        // ── Legacy services (CIS 2.x) ─────────────────────────────────────
        SecurityRule {
            id: "CIS-2.1".into(),
            name: "Remove legacy services (telnet, rsh, rcp)".into(),
            description: "Legacy services such as telnet, rsh, and rcp should be removed or disabled as they transmit data in cleartext.".into(),
            severity: Severity::Critical,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "not installed".into(),
            remediation_steps: "Uninstall legacy packages: apt remove telnetd rsh-server rsh-client rcp".into(),
        },
        SecurityRule {
            id: "CIS-2.2".into(),
            name: "Remove X Window System".into(),
            description: "The X Window System should not be installed on servers as it adds unnecessary attack surface.".into(),
            severity: Severity::Low,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "not installed".into(),
            remediation_steps: "Uninstall X Window packages: apt remove xserver-xorg".into(),
        },
        // ── Daemon configuration (CIS 3.x) ────────────────────────────────
        SecurityRule {
            id: "CIS-3.1".into(),
            name: "Set daemon umask to 027 or stricter".into(),
            description: "The default umask for daemons should be set to 027 or stricter to restrict file permissions.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "027".into(),
            remediation_steps: "Set 'umask 027' in /etc/init.d/functions or the relevant systemd service override.".into(),
        },
        // ── Core dumps (CIS 4.x) ──────────────────────────────────────────
        SecurityRule {
            id: "CIS-4.1".into(),
            name: "Disable core dumps".into(),
            description: "Core dumps should be disabled to prevent sensitive data from being written to disk in crash dumps.".into(),
            severity: Severity::Medium,
            category: "kernel".into(),
            check_type: "cis_linux".into(),
            expected_value: "* hard core 0".into(),
            remediation_steps: "Add '* hard core 0' and '* soft core 0' to /etc/security/limits.conf and /etc/sysctl.conf: fs.suid_dumpable=0".into(),
        },
        SecurityRule {
            id: "CIS-4.2".into(),
            name: "Set sysctl fs.suid_dumpable = 0".into(),
            description: "The sysctl parameter fs.suid_dumpable should be set to 0 to prevent SUID programs from producing core dumps.".into(),
            severity: Severity::Medium,
            category: "kernel".into(),
            check_type: "cis_linux".into(),
            expected_value: "0".into(),
            remediation_steps: "Add 'fs.suid_dumpable = 0' to /etc/sysctl.conf and run: sysctl -p".into(),
        },
        // ── Logging (CIS 5.x) ─────────────────────────────────────────────
        SecurityRule {
            id: "CIS-5.1".into(),
            name: "Configure rsyslog".into(),
            description: "Rsyslog should be installed, enabled, and configured to send logs to a central log server.".into(),
            severity: Severity::High,
            category: "network".into(),
            check_type: "cis_linux".into(),
            expected_value: "enabled and configured".into(),
            remediation_steps: "Install rsyslog if not present, enable it (systemctl enable rsyslog), and configure remote logging in /etc/rsyslog.conf.".into(),
        },
        SecurityRule {
            id: "CIS-5.2".into(),
            name: "Configure file permissions for logs".into(),
            description: "Log files should have appropriate permissions (640 or stricter) to prevent unauthorized access.".into(),
            severity: Severity::Medium,
            category: "filesystem".into(),
            check_type: "cis_linux".into(),
            expected_value: "640".into(),
            remediation_steps: "Set permissions: chmod 640 /var/log/syslog /var/log/auth.log and ensure proper ownership by syslog:adm.".into(),
        },
        // ── SSH hardening ──────────────────────────────────────────────────
        SecurityRule {
            id: "SSH-1".into(),
            name: "SSH: PermitRootLogin no".into(),
            description: "Direct root login via SSH should be disabled to enforce least-privilege access.".into(),
            severity: Severity::Critical,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "no".into(),
            remediation_steps: "Set 'PermitRootLogin no' in /etc/ssh/sshd_config and restart sshd.".into(),
        },
        SecurityRule {
            id: "SSH-2".into(),
            name: "SSH: Protocol 2 only".into(),
            description: "SSH should only use Protocol 2 for secure key exchange and encryption.".into(),
            severity: Severity::High,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "2".into(),
            remediation_steps: "Set 'Protocol 2' in /etc/ssh/sshd_config and restart sshd.".into(),
        },
        SecurityRule {
            id: "SSH-3".into(),
            name: "SSH: MaxAuthTries <= 4".into(),
            description: "Maximum authentication attempts should be limited to 4 or fewer to mitigate brute-force attacks.".into(),
            severity: Severity::Medium,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "4".into(),
            remediation_steps: "Set 'MaxAuthTries 4' in /etc/ssh/sshd_config and restart sshd.".into(),
        },
        SecurityRule {
            id: "SSH-4".into(),
            name: "SSH: Disable empty passwords".into(),
            description: "Empty password authentication should be disabled to prevent unauthorized access.".into(),
            severity: Severity::Critical,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "no".into(),
            remediation_steps: "Set 'PermitEmptyPasswords no' in /etc/ssh/sshd_config and restart sshd.".into(),
        },
        SecurityRule {
            id: "SSH-5".into(),
            name: "SSH: Disable X11Forwarding".into(),
            description: "X11 forwarding should be disabled on servers to reduce the attack surface.".into(),
            severity: Severity::Low,
            category: "auth".into(),
            check_type: "cis_linux".into(),
            expected_value: "no".into(),
            remediation_steps: "Set 'X11Forwarding no' in /etc/ssh/sshd_config and restart sshd.".into(),
        },
        // ── Docker security ────────────────────────────────────────────────
        SecurityRule {
            id: "DOCKER-1".into(),
            name: "Docker: No privileged containers".into(),
            description: "Containers should not run in privileged mode as it grants full host capabilities.".into(),
            severity: Severity::Critical,
            category: "docker".into(),
            check_type: "cis_docker".into(),
            expected_value: "false".into(),
            remediation_steps: "Remove --privileged flag from container run commands. Use specific --cap-add instead.".into(),
        },
        SecurityRule {
            id: "DOCKER-2".into(),
            name: "Docker: Audit docker daemon".into(),
            description: "The Docker daemon should be audited to track configuration changes and access.".into(),
            severity: Severity::High,
            category: "docker".into(),
            check_type: "cis_docker".into(),
            expected_value: "audit enabled".into(),
            remediation_steps: "Add audit rules for Docker daemon files: -w /usr/bin/docker -p wa -k docker-daemon".into(),
        },
        SecurityRule {
            id: "DOCKER-3".into(),
            name: "Docker: Default ulimit configured".into(),
            description: "Default ulimits for containers should be explicitly configured to prevent resource exhaustion.".into(),
            severity: Severity::Medium,
            category: "docker".into(),
            check_type: "cis_docker".into(),
            expected_value: "explicit ulimits".into(),
            remediation_steps: "Set default ulimits in /etc/docker/daemon.json with appropriate nofile and nproc limits.".into(),
        },
        SecurityRule {
            id: "DOCKER-4".into(),
            name: "Docker: User namespace remapping enabled".into(),
            description: "User namespace remapping should be enabled to isolate container user IDs from host.".into(),
            severity: Severity::High,
            category: "docker".into(),
            check_type: "cis_docker".into(),
            expected_value: "enabled".into(),
            remediation_steps: "Set 'userns-remap: default' in /etc/docker/daemon.json and restart Docker.".into(),
        },
        SecurityRule {
            id: "DOCKER-5".into(),
            name: "Docker: Content trust enabled".into(),
            description: "Docker Content Trust should be enabled to ensure image integrity via digital signatures.".into(),
            severity: Severity::Medium,
            category: "docker".into(),
            check_type: "cis_docker".into(),
            expected_value: "DOCKER_CONTENT_TRUST=1".into(),
            remediation_steps: "Set environment variable: export DOCKER_CONTENT_TRUST=1 in Docker daemon environment.".into(),
        },
        // ── Vulnerability checks ───────────────────────────────────────────
        SecurityRule {
            id: "VULN-1".into(),
            name: "Check for known CVEs in installed packages".into(),
            description: "All installed packages should be scanned against known CVE databases and patched as needed.".into(),
            severity: Severity::High,
            category: "vulnerability".into(),
            check_type: "vulnerability".into(),
            expected_value: "no known CVEs".into(),
            remediation_steps: "Run 'apt update && apt upgrade' or 'yum update' to patch known vulnerabilities.".into(),
        },
        SecurityRule {
            id: "VULN-2".into(),
            name: "Check for outdated OpenSSL".into(),
            description: "OpenSSL should be at the latest stable version to avoid known cryptographic vulnerabilities.".into(),
            severity: Severity::Critical,
            category: "vulnerability".into(),
            check_type: "vulnerability".into(),
            expected_value: "latest stable".into(),
            remediation_steps: "Update OpenSSL: apt install --only-upgrade openssl".into(),
        },
        SecurityRule {
            id: "VULN-3".into(),
            name: "Check for weak SSL/TLS protocols".into(),
            description: "TLS 1.0 and 1.1 should be disabled in favor of TLS 1.2+ to prevent known protocol attacks.".into(),
            severity: Severity::High,
            category: "vulnerability".into(),
            check_type: "vulnerability".into(),
            expected_value: "TLS 1.2+".into(),
            remediation_steps: "Configure services to disable TLS 1.0/1.1. Set SSLProtocol to 'all -SSLv2 -SSLv3 -TLSv1 -TLSv1.1' in Apache/Nginx.".into(),
        },
        SecurityRule {
            id: "VULN-4".into(),
            name: "Check for weak ciphers".into(),
            description: "Weak ciphers (RC4, DES, 3DES) should be disabled to ensure strong encryption.".into(),
            severity: Severity::High,
            category: "vulnerability".into(),
            check_type: "vulnerability".into(),
            expected_value: "AES-GCM, ChaCha20".into(),
            remediation_steps: "Set SSLCipherSuite to exclude weak ciphers. Use Mozilla SSL Configuration Generator for recommended settings.".into(),
        },
        // ── Patch management ───────────────────────────────────────────────
        SecurityRule {
            id: "PATCH-1".into(),
            name: "Automatic security updates enabled".into(),
            description: "Unattended security updates should be enabled to ensure timely patching.".into(),
            severity: Severity::Medium,
            category: "patch".into(),
            check_type: "patch".into(),
            expected_value: "enabled".into(),
            remediation_steps: "Install and configure unattended-upgrades: apt install unattended-upgrades && dpkg-reconfigure unattended-upgrades".into(),
        },
        SecurityRule {
            id: "PATCH-2".into(),
            name: "Check for pending security patches".into(),
            description: "Pending security patches should be applied promptly to reduce vulnerability exposure window.".into(),
            severity: Severity::High,
            category: "patch".into(),
            check_type: "patch".into(),
            expected_value: "0 pending".into(),
            remediation_steps: "Apply pending patches: apt update && apt upgrade -y".into(),
        },
    ]
}
