use zbus::proxy;

#[proxy(
    interface = "org.vibe.Daemon",
    default_service = "org.vibe.Daemon",
    default_path = "/org/vibe/Daemon"
)]
pub trait VibeControl {
    async fn next(&self) -> zbus::Result<()>;
    async fn pause(&self) -> zbus::Result<()>;
    async fn resume(&self) -> zbus::Result<()>;
    async fn reload_config(&self) -> zbus::Result<()>;
    async fn status(&self) -> zbus::Result<String>;
    async fn favorite(&self) -> zbus::Result<()>;
}
