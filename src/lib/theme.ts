import { ref } from "vue";

export type ThemeMode = "light" | "dark" | "system";

const STORAGE_KEY = "svnm-theme";
const media = window.matchMedia("(prefers-color-scheme: dark)");

function load(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* storage unavailable */
  }
  return "system";
}

export const themeMode = ref<ThemeMode>(load());

function apply() {
  const dark = themeMode.value === "dark" || (themeMode.value === "system" && media.matches);
  document.documentElement.classList.toggle("dark", dark);
}

export function setTheme(mode: ThemeMode) {
  themeMode.value = mode;
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* storage unavailable */
  }
  apply();
}

export function initTheme() {
  apply();
  media.addEventListener("change", apply);
  // macOS gets the translucent "glass" treatment (window vibrancy shows
  // through); other platforms keep solid surfaces.
  if (navigator.userAgent.includes("Mac")) {
    document.documentElement.classList.add("glass");
  }
}

export function formatDate(iso: string): string {
  const d = new Date(iso);
  return isNaN(d.getTime())
    ? iso
    : d.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}
