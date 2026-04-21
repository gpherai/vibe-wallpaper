# Vibe

Vibe is een lichte Linux wallpaper manager met quotes.  
De app draait als daemon op de achtergrond en is aan te sturen via een CLI en een Tauri GUI.

## Onderdelen

- `vibe-core`: gedeelde logica (providers, compositing, desktop integratie, configuratie)
- `vibe-daemon`: achtergrondproces met D-Bus service (`org.vibe.Daemon`)
- `vibe-cli`: command-line client voor daemonbediening
- `gui` (`vibe-gui`): Tauri desktop interface

## Vereisten

Rust toolchain + Linux desktop dependencies (Tauri/GTK):

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
# Fedora:
# sudo dnf install webkit2gtk4.1-devel cairo-devel pango-devel atk-devel gdk-pixbuf2-devel
```

## Bouwen

Vanuit project root:

```bash
cargo build --release
```

Optioneel binaries installeren in `~/.local/bin`:

```bash
cargo install --path daemon --root ~/.local
cargo install --path cli --root ~/.local
```

## Starten

Daemon:

```bash
cargo run --bin vibe-daemon
```

GUI (development):

```bash
cd gui
npm install
npm run tauri dev
```

## CLI gebruik

```bash
cargo run --bin vibe-cli -- status
cargo run --bin vibe-cli -- next
cargo run --bin vibe-cli -- pause
cargo run --bin vibe-cli -- resume
cargo run --bin vibe-cli -- reload-config
```

## Configuratie

Configbestand:

```text
~/.config/vibe/config.toml
```

Voorbeeld:

```toml
wallpaper_interval_mins = 60
quote_interval_mins = 60
subreddit = "EarthPorn"
provider_type = "reddit"      # reddit | unsplash | earthview
unsplash_access_key = ""      # verplicht bij provider_type = "unsplash"
unsplash_query = "nature,wallpapers"
quote_provider_type = "zenquotes"  # zenquotes | localfile
quote_local_path = ""
is_paused = false
```

Na handmatige wijziging van `config.toml`:

```bash
cargo run --bin vibe-cli -- reload-config
```

De GUI gebruikt dezelfde config en kan die ook direct opslaan/herladen.

## systemd user service (optioneel)

Er staat een voorbeeldservice in `daemon/vibe.service`.  
Installeer eerst `vibe-daemon` naar `~/.local/bin`, kopieer daarna de service naar `~/.config/systemd/user/` en activeer hem:

```bash
mkdir -p ~/.config/systemd/user
cp daemon/vibe.service ~/.config/systemd/user/vibe.service
systemctl --user daemon-reload
systemctl --user enable --now vibe.service
```
