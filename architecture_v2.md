# Vibe Architectuur & Ontwerp (V2)

## 1. Concept & Doel
Vibe is een premium, moderne en resource-efficiënte Linux-applicatie voor Fedora (GNOME/KDE) die automatisch wallpapers en quotes ophaalt, deze prachtig samenvoegt, en instelt als bureaubladachtergrond. Het draait geruisloos op de achtergrond en biedt volledige controle via zowel een snelle CLI als een moderne GUI.

## 2. Moderne Software Engineering Principes
- **Decoupling (Ontkoppeling):** De applicatie is opgesplitst in een onafhankelijke Daemon (achtergronddienst), een CLI (voor snelle commando's) en een GUI (voor visuele instellingen). Ze communiceren via standaard Linux IPC (Inter-Process Communication).
- **Modulair & Extensibel:** De `vibe-core` library bevat alle herbruikbare logica (providers, compositor, portal integratie). Nieuwe afbeeldingsbronnen toevoegen vereist slechts een implementatie van de `WallpaperProvider` trait.
- **Native Integratie:** We gebruiken `ashpd` (XDG Desktop Portals) voor universele en toekomstbestendige desktop integratie (werkt op Wayland, GNOME, KDE, etc.). Voor IPC gebruiken we `zbus` om een native D-Bus service op te zetten.
- **Resource Efficiëntie:** De backend is geschreven in Rust. De daemon slaapt 99% van de tijd en verbruikt nagenoeg geen CPU of RAM. De GUI is gebouwd met Tauri v2, wat resulteert in een extreem lichte frontend (geen zware Chromium/Electron overhead).

## 3. Systeemcomponenten

### 3.1. Vibe Core (`vibe-core` crate)
De ruggengraat van de applicatie.
- **Providers:** Verantwoordelijk voor het ophalen van data (`reqwest`, asynchroon via `tokio`).
  - *Wallpaper:* Reddit API (hot images filtering).
  - *Quotes:* ZenQuotes API.
- **Compositor:** Voegt afbeeldingen en tekst samen met professionele typografie (anti-aliasing, drop shadows) met behulp van `imageproc` en `ab_glyph`.
- **Desktop Adapter:** Stelt de wallpaper in via de `ashpd` crate (XDG Desktop Portal).
- **Configuratie:** Laadt instellingen (bijv. intervallen, actieve bronnen) uit `~/.config/vibe/config.toml` (met `serde`).

### 3.2. Vibe Daemon (`vibed`)
Een `systemd` user service die op de achtergrond draait.
- Bevat de hoofd-lus (`tokio::time::sleep` op basis van ingestelde intervallen).
- Zet een **D-Bus Service** op via de `zbus` crate (`org.vibe.Daemon`).
- Exposeert methodes zoals: `next()`, `pause()`, `resume()`, `status()`, en `reload_config()`.

### 3.3. Vibe CLI (`vibectl`)
Een command-line interface gebouwd met `clap`.
- Reageert onmiddellijk doordat het als een **D-Bus Client** (via `zbus` proxy) communiceert met de daemon.
- Commando's: `vibectl next`, `vibectl pause`, `vibectl status`.

### 3.4. Vibe GUI (`vibe-gui`)
De grafische gebruikersinterface, gebouwd met **Tauri v2**.
- **Backend:** Rust (Tauri commands). Fungeert ook als D-Bus client om de daemon aan te sturen.
- **Frontend:** HTML/TypeScript met **Vanilla CSS** (zoals aanbevolen voor maximale controle en prestaties, zonder externe afhankelijkheden zoals Tailwind).
- **Functies:** Visuele weergave van de huidige status, knoppen voor 'Next/Pause/Resume', en een formulier om instellingen te wijzigen (waarna `reload_config()` naar de daemon wordt gestuurd).

## 4. Datastroom
1. De gebruiker start de computer op. `systemd` start `vibed`.
2. `vibed` leest de configuratie, haalt direct een wallpaper + quote op, bewerkt deze en roept de Desktop Portal aan.
3. `vibed` wacht (bijv. 60 minuten) en adverteert zijn aanwezigheid op de D-Bus (`org.vibe.Daemon`).
4. De gebruiker opent de GUI of gebruikt de CLI (`vibectl next`).
5. Dit stuurt een D-Bus signaal naar de daemon.
6. De daemon onderbreekt zijn slaapstand, haalt direct nieuwe data op, update de wallpaper en reset de timer.

## 5. Ontwikkelingsplan (Fasering)
Dit plan zal strikt gevolgd worden voor de implementatie:

- **Fase 1: Core & D-Bus architectuur**
  - Config-module implementeren (lezen/schrijven van `~/.config/vibe/config.toml`).
  - Toevoegen van `zbus` aan de `vibe-core` en `vibe-daemon`.
  - Opzetten van de D-Bus service in de daemon.
- **Fase 2: CLI Implementatie**
  - Genereren van de D-Bus proxy via `zbus`.
  - Koppelen van de `clap` commando's in `vibe-cli` aan de D-Bus methodes.
- **Fase 3: Daemon Logica & State Management**
  - Implementeren van de daadwerkelijke control-flow in de daemon (Pause/Resume state, dynamische intervallen op basis van config).
- **Fase 4: Tauri GUI Ontwikkeling**
  - Initialiseren van een Tauri v2 project binnen de workspace.
  - Koppelen van Tauri backend aan D-Bus proxy.
  - Bouwen van een strakke, moderne Vanilla CSS / TypeScript frontend.
- **Fase 5: Testen & Polish**
  - Integratietesten, error handling optimaliseren (netwerkuitval opvangen), en documentatie.
