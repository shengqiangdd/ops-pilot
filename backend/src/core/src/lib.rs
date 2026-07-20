//! # ops-pilot-core
//!
//! 核心基础设施层，提供 OpsPilot 平台的底层能力。
//!
//! 本 crate 是所有上层模块（mod-core、mod-rca、mod-security 等）和网关的基石，
//! 封装了与外部系统交互的客户端、数据模型和业务逻辑。
//!
//! ## 模块概览
//!
//! | 模块 | 职责 |
//! |------|------|
//! | [`ssh`] | SSH 连接池、远程命令执行、会话管理 |
//! | [`docker`] | Docker 容器生命周期管理（基于 bollard） |
//! | [`host`] | 主机注册、配置存储、凭据加密 |
//! | [`monitor`] | 主机指标采集与健康检查 |
//! | [`auth`] | 用户注册、登录、JWT 签发与验证 |
//! | [`vault`] | Vault 密钥缓存，支持按用户加密/解密 |
//! | [`crypto`] | AES-256-GCM 加密/解密，主密钥管理 |
//! | [`db`] | SQLite 数据库连接池与迁移 |
//! | [`alert`] | 规则引擎驱动的异常检测（夜批、高失败率、首次连接） |
//! | [`audit`] | 审计日志写入与事件发布 |

pub mod alert;
pub mod audit;
pub mod auth;
pub mod crypto;
pub mod db;
pub mod docker;
pub mod host;
pub mod monitor;
pub mod ssh;
pub mod vault;
