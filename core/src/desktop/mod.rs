pub mod portal;

pub trait DesktopEnvironment {
    fn set_wallpaper(
        &self,
        path: &std::path::Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::Result<()>> + Send + '_>>;
}

pub fn get_current_desktop() -> Box<dyn DesktopEnvironment> {
    // We use the modern XDG Desktop Portal which works on GNOME, KDE, and wlroots (Wayland/X11)
    Box::new(portal::PortalAdapter)
}
