pub mod agent;
pub mod audit;
pub mod hosts;
pub mod modules;
pub mod security;
pub mod vault;

pub use crate::terminal::terminal_routes;
pub use crate::ws_events::ws_events_handler;
pub use agent::agent_routes;
pub use audit::audit_routes;
pub use hosts::host_routes;
pub use modules::module_routes;
pub use security::security_routes;
pub use vault::vault_routes;
