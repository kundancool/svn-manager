import { defineStore } from "pinia";

// User preferences — frontend concerns, persisted in localStorage.
// (Backend settings like the svn path live in the app config file.)

export type DiffTheme = "monokai" | "app" | "github" | "dracula" | "solarized";

export interface Prefs {
  showUnversioned: boolean;
  confirmDestructive: boolean;
  diffFontSize: "small" | "medium" | "large";
  historyPageSize: 25 | 50 | 100;
  diffTheme: DiffTheme;
  diffWrap: boolean;
}

const KEY = "svnm-prefs";

const defaults: Prefs = {
  showUnversioned: true,
  confirmDestructive: true,
  diffFontSize: "medium",
  historyPageSize: 50,
  diffTheme: "monokai",
  diffWrap: false,
};

function load(): Prefs {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return { ...defaults, ...JSON.parse(raw) };
  } catch {
    /* storage unavailable or corrupt */
  }
  return { ...defaults };
}

export const usePrefsStore = defineStore("prefs", {
  state: (): Prefs => load(),
  getters: {
    diffFontPx: (s) => ({ small: 11, medium: 12, large: 13.5 })[s.diffFontSize],
  },
  actions: {
    set<K extends keyof Prefs>(key: K, value: Prefs[K]) {
      // @ts-expect-error index write on typed state
      this[key] = value;
      try {
        const { showUnversioned, confirmDestructive, diffFontSize, historyPageSize, diffTheme, diffWrap } = this;
        localStorage.setItem(
          KEY,
          JSON.stringify({ showUnversioned, confirmDestructive, diffFontSize, historyPageSize, diffTheme, diffWrap })
        );
      } catch {
        /* storage unavailable */
      }
    },
  },
});
