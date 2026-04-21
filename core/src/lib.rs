pub mod compositor;
pub mod config;
pub mod desktop;
pub mod ipc;
pub mod providers;
pub mod scheduler;

pub type Result<T> = std::result::Result<T, anyhow::Error>;
