pub mod agent;
pub mod hosts;
pub mod modules;

pub use agent::agent_routes;
pub use crate::terminal::terminal_routes;
pub use hosts::host_routes;
pub use modules::module_routes;
