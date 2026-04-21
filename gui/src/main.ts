import { invoke } from "@tauri-apps/api/core";

const HIDDEN_CLASS = "is-hidden";

let statusIndicator: HTMLElement | null;
let btnFavorite: HTMLButtonElement | null;
let btnNext: HTMLButtonElement | null;
let btnPause: HTMLButtonElement | null;
let btnResume: HTMLButtonElement | null;
let configForm: HTMLFormElement | null;
let inputInterval: HTMLInputElement | null;
let inputQuoteInterval: HTMLInputElement | null;
let inputSubreddit: HTMLInputElement | null;
let selectProvider: HTMLSelectElement | null;
let inputUnsplashKey: HTMLInputElement | null;
let inputUnsplashQuery: HTMLInputElement | null;
let groupSubreddit: HTMLElement | null;
let groupUnsplash: HTMLElement | null;
let selectQuoteProvider: HTMLSelectElement | null;
let groupQuotePath: HTMLElement | null;
let inputQuotePath: HTMLInputElement | null;
let formFeedback: HTMLElement | null;

interface AppConfig {
  wallpaper_interval_mins: number;
  quote_interval_mins: number;
  subreddit: string;
  provider_type: "reddit" | "unsplash" | "earthview" | "bing" | "wallhaven";
  unsplash_access_key: string | null;
  unsplash_query: string | null;
  quote_provider_type: "zenquotes" | "localfile";
  quote_local_path: string | null;
  is_paused: boolean;
}

function showFeedback(message: string, type: "success" | "error") {
  if (!formFeedback) return;
  formFeedback.textContent = message;
  formFeedback.className = `feedback-message ${type}`;
  toggleHidden(formFeedback, false);
  
  // Hide success messages after 5 seconds
  if (type === "success") {
    setTimeout(() => {
      if (formFeedback?.textContent === message) {
        toggleHidden(formFeedback, true);
      }
    }, 5000);
  }
}

function toggleHidden(element: HTMLElement | null, hidden: boolean) {
  if (!element) return;
  element.classList.toggle(HIDDEN_CLASS, hidden);
}

function parseMinutes(value: string, fallback: number): number {
  const parsed = Number.parseInt(value, 10);
  if (Number.isFinite(parsed) && parsed > 0) {
    return parsed;
  }
  return fallback;
}

function setControlsDisabled(disabled: boolean) {
  [btnFavorite, btnNext, btnPause, btnResume].forEach(btn => {
    if (btn) btn.disabled = disabled;
  });
}

async function fetchStatus() {
  if (!statusIndicator) return;
  try {
    const status = await invoke<string>("get_status");
    statusIndicator.textContent = status;
    statusIndicator.style.color = "";
    
    setControlsDisabled(false);

    const paused = status.toLowerCase().includes("paused");
    toggleHidden(btnPause, paused);
    toggleHidden(btnResume, !paused);
  } catch (error) {
    console.error("Failed to fetch status:", error);
    statusIndicator.textContent = "Error: Cannot connect to daemon.";
    statusIndicator.style.color = "var(--danger)";
    
    // Disable controls if daemon is unreachable
    setControlsDisabled(true);
    toggleHidden(btnPause, false);
    toggleHidden(btnResume, true);
  } finally {
    // Schedule next poll
    setTimeout(fetchStatus, 5000);
  }
}

async function fetchConfig() {
  try {
    const config = await invoke<AppConfig>("get_config");
    if (inputInterval) inputInterval.value = config.wallpaper_interval_mins.toString();
    if (inputQuoteInterval) inputQuoteInterval.value = config.quote_interval_mins.toString();
    if (inputSubreddit) inputSubreddit.value = config.subreddit;
    if (selectProvider) {
      selectProvider.value = config.provider_type;
      updateFieldVisibility(config.provider_type);
    }
    if (inputUnsplashKey) inputUnsplashKey.value = config.unsplash_access_key ?? "";
    if (inputUnsplashQuery) inputUnsplashQuery.value = config.unsplash_query ?? "";
    if (selectQuoteProvider) {
      selectQuoteProvider.value = config.quote_provider_type;
      updateQuoteFieldVisibility(config.quote_provider_type);
    }
    if (inputQuotePath) inputQuotePath.value = config.quote_local_path ?? "";
  } catch (error) {
    console.error("Failed to load config:", error);
    showFeedback("Failed to load config. Check ~/.config/vibe/config.toml", "error");
  }
}

function updateFieldVisibility(provider: AppConfig["provider_type"] | string) {
  toggleHidden(groupSubreddit, provider !== "reddit");
  toggleHidden(groupUnsplash, provider !== "unsplash");
}

function updateQuoteFieldVisibility(provider: AppConfig["quote_provider_type"] | string) {
  toggleHidden(groupQuotePath, provider !== "localfile");
}

async function saveConfig(e: Event) {
  e.preventDefault();
  toggleHidden(formFeedback, true);

  if (
    !inputInterval ||
    !inputQuoteInterval ||
    !inputSubreddit ||
    !selectProvider ||
    !inputUnsplashKey ||
    !inputUnsplashQuery ||
    !selectQuoteProvider ||
    !inputQuotePath
  ) {
    return;
  }

  const currentConfig = await invoke<AppConfig>("get_config").catch(() => null);
  if (!currentConfig) {
    showFeedback("Failed to load current config. Is the daemon running?", "error");
    return;
  }

  const providerType = selectProvider.value as AppConfig["provider_type"];
  const quoteProviderType = selectQuoteProvider.value as AppConfig["quote_provider_type"];
  const quotePath = inputQuotePath.value.trim();
  const unsplashKey = inputUnsplashKey.value.trim();

  if (providerType === "unsplash" && unsplashKey.length === 0) {
    showFeedback("Please provide an Unsplash access key.", "error");
    return;
  }

  if (quoteProviderType === "localfile" && quotePath.length === 0) {
    showFeedback("Please provide a local quote file path.", "error");
    return;
  }

  const newConfig: AppConfig = {
    ...currentConfig,
    wallpaper_interval_mins: parseMinutes(inputInterval.value, 60),
    quote_interval_mins: parseMinutes(inputQuoteInterval.value, 60),
    subreddit: inputSubreddit.value.trim() || "EarthPorn",
    provider_type: providerType,
    unsplash_access_key: unsplashKey || null,
    unsplash_query: inputUnsplashQuery.value.trim() || null,
    quote_provider_type: quoteProviderType,
    quote_local_path: quotePath || null,
  };

  try {
    await invoke("save_config", { config: newConfig });

    try {
      await invoke("reload_config");
      if (!newConfig.is_paused) {
        await invoke("next_wallpaper");
      }
      showFeedback("Configuration saved and reloaded.", "success");
    } catch (reloadError) {
      console.error("Config reload failed:", reloadError);
      showFeedback("Config saved, but failed to notify daemon.", "error");
    }

    await fetchStatus();
  } catch (error) {
    console.error("Failed to save config:", error);
    showFeedback("Error saving configuration: " + error, "error");
  }
}

async function nextWallpaper() {
  try {
    await invoke("next_wallpaper");
    await fetchStatus();
  } catch (error) {
    console.error(error);
    showFeedback("Failed to skip wallpaper.", "error");
  }
}

async function pauseWallpaper() {
  try {
    await invoke("pause_wallpaper");
    await fetchStatus();
  } catch (error) {
    console.error(error);
    showFeedback("Failed to pause rotation.", "error");
  }
}

async function resumeWallpaper() {
  try {
    await invoke("resume_wallpaper");
    await fetchStatus();
  } catch (error) {
    console.error(error);
    showFeedback("Failed to resume rotation.", "error");
  }
}

async function favoriteWallpaper() {
  try {
    await invoke("favorite_wallpaper");
    showFeedback("Wallpaper saved to favorites!", "success");
  } catch (error) {
    console.error(error);
    showFeedback("Failed to save favorite.", "error");
  }
}

window.addEventListener("DOMContentLoaded", () => {
  statusIndicator = document.getElementById("status-indicator");
  btnFavorite = document.getElementById("btn-favorite") as HTMLButtonElement;
  btnNext = document.getElementById("btn-next") as HTMLButtonElement;
  btnPause = document.getElementById("btn-pause") as HTMLButtonElement;
  btnResume = document.getElementById("btn-resume") as HTMLButtonElement;
  configForm = document.getElementById("config-form") as HTMLFormElement;
  inputInterval = document.getElementById("wallpaper-interval") as HTMLInputElement;
  inputQuoteInterval = document.getElementById("quote-interval") as HTMLInputElement;
  inputSubreddit = document.getElementById("subreddit") as HTMLInputElement;
  selectProvider = document.getElementById("provider-type") as HTMLSelectElement;
  inputUnsplashKey = document.getElementById("unsplash-key") as HTMLInputElement;
  inputUnsplashQuery = document.getElementById("unsplash-query") as HTMLInputElement;
  groupSubreddit = document.getElementById("group-subreddit");
  groupUnsplash = document.getElementById("group-unsplash");
  selectQuoteProvider = document.getElementById("quote-provider-type") as HTMLSelectElement;
  groupQuotePath = document.getElementById("group-quote-path");
  inputQuotePath = document.getElementById("quote-path") as HTMLInputElement;
  formFeedback = document.getElementById("form-feedback");

  if (btnFavorite) btnFavorite.addEventListener("click", favoriteWallpaper);
  if (btnNext) btnNext.addEventListener("click", nextWallpaper);
  if (btnPause) btnPause.addEventListener("click", pauseWallpaper);
  if (btnResume) btnResume.addEventListener("click", resumeWallpaper);
  if (configForm) configForm.addEventListener("submit", saveConfig);
  if (selectProvider) {
    selectProvider.addEventListener("change", (e) => {
      updateFieldVisibility((e.target as HTMLSelectElement).value);
    });
  }
  if (selectQuoteProvider) {
    selectQuoteProvider.addEventListener("change", (e) => {
      updateQuoteFieldVisibility((e.target as HTMLSelectElement).value);
    });
  }

  const tabBtns = document.querySelectorAll('.tab-btn');
  const tabContents = document.querySelectorAll('.tab-content');

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('aria-controls');
      if (!targetId) return;

      tabBtns.forEach(b => {
        b.classList.remove('active');
        b.setAttribute('aria-selected', 'false');
      });
      tabContents.forEach(c => c.classList.remove('active'));

      btn.classList.add('active');
      btn.setAttribute('aria-selected', 'true');
      
      const targetContent = document.getElementById(targetId);
      if (targetContent) {
        targetContent.classList.add('active');
      }
    });
  });

  fetchStatus();
  fetchConfig();
});
