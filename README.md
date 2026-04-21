# Vibe - Linux Desktop Aesthetic Manager

Vibe is a modular, lightweight Linux wallpaper manager built with Rust. It automatically fetches beautiful wallpapers and inspiring quotes, composites them together with a sleek drop-shadow effect, and sets the result as your desktop background.

It is designed with a decoupled architecture, meaning the background service (Daemon), the Command-Line Interface (CLI), and the Graphical User Interface (GUI) operate independently and communicate seamlessly via D-Bus.

## Features

- **Wallpaper Providers:** Reddit (e.g., EarthPorn), Unsplash, Google Earth View.
- **Quote Providers:** ZenQuotes API (built-in, no key required), Local Text Files.
- **Smart Compositing:** Automatically converts images and applies quotes with readable typography and soft drop-shadows. Falls back to system fonts (`fc-match`) automatically.
- **D-Bus Integration:** Control the background daemon instantly via CLI or GUI.
- **Modern GUI:** Built with Tauri v2, Vite, TypeScript, and modern CSS (`oklch` relative colors) for a sleek, native-feeling dashboard.
- **Wayland & X11 Support:** Uses `ashpd` (XDG Desktop Portals) for broad compatibility across modern Linux desktop environments like GNOME and KDE.

## Project Structure

- `core/` (`vibe-core`): Shared business logic, provider traits, image compositing, and XDG desktop integration.
- `daemon/` (`vibe-daemon`): The background service managing the fetch-and-set cycle and exposing the `org.vibe.Daemon` D-Bus interface.
- `cli/` (`vibe-cli`): A fast command-line client to control the daemon.
- `gui/` (`vibe-gui`): A Tauri-based dashboard to manage configuration and control rotation.

---

## Prerequisites

You need the Rust toolchain, Node.js/npm, and Linux desktop development dependencies (for Tauri and GTK).

**Ubuntu / Debian:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Fedora:**
```bash
sudo dnf install webkit2gtk4.1-devel cairo-devel pango-devel atk-devel gdk-pixbuf2-devel
```

---

## Getting Started

Because Vibe uses a decoupled architecture, you must run the **Daemon** to handle the heavy lifting, while you can use the **GUI** or **CLI** to control it.

### 1. Start the Daemon

The daemon must be running in the background for the CLI and GUI to work properly. From the project root, open a terminal and run:

```bash
cargo run --bin vibe-daemon
```

### 2. Launch the GUI (Dashboard)

In a **separate terminal**, navigate to the `gui` folder and start the Tauri development server:

```bash
cd gui
npm install
npm run tauri dev
```
*The GUI will automatically connect to the daemon, allowing you to configure intervals, choose providers, and skip/pause wallpapers.*

### 3. Using the CLI

You can also control the daemon directly from your terminal using the CLI:

```bash
cargo run --bin vibe-cli -- status
cargo run --bin vibe-cli -- next
cargo run --bin vibe-cli -- pause
cargo run --bin vibe-cli -- resume
cargo run --bin vibe-cli -- reload
```

---

## Configuration

Vibe stores its configuration at `~/.config/vibe/config.toml`. The GUI will manage this file for you, but you can also edit it manually.

Example `config.toml`:

```toml
wallpaper_interval_mins = 60
quote_interval_mins = 60
subreddit = "EarthPorn"
provider_type = "earthview"      # reddit | unsplash | earthview
unsplash_access_key = ""         # required if provider_type = "unsplash"
unsplash_query = "nature,wallpapers"
quote_provider_type = "zenquotes" # zenquotes | localfile
quote_local_path = ""             # required if quote_provider_type = "localfile"
is_paused = false
```

If you modify the file manually while the daemon is running, apply the changes via the CLI:
```bash
cargo run --bin vibe-cli -- reload
```

---

## Systemd User Service (Optional)

For permanent installation, you can set Vibe to start automatically on login.

1. Build and install the binaries:
   ```bash
   cargo install --path daemon --root ~/.local
   cargo install --path cli --root ~/.local
   ```
2. Set up the systemd service:
   ```bash
   mkdir -p ~/.config/systemd/user
   cp daemon/vibe.service ~/.config/systemd/user/vibe.service
   systemctl --user daemon-reload
   systemctl --user enable --now vibe.service
   ```
