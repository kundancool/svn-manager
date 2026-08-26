import { defineStore } from "pinia";
import { api, errorMessage } from "@/lib/api";
import type {
  AppConfig,
  OpenedProject,
  RepoLayout,
  StatusEntry,
  SvnBinary,
} from "@/lib/types";

export type ViewId = "changes" | "history" | "branches" | "browser" | "publish";

export interface Toast {
  id: number;
  tone: "ok" | "error" | "info";
  text: string;
}

let toastSeq = 0;

export const useAppStore = defineStore("app", {
  state: () => ({
    svn: null as SvnBinary | null,
    svnChecked: false,
    config: null as AppConfig | null,
    project: null as OpenedProject | null,
    layout: null as RepoLayout | null,
    view: "changes" as ViewId,
    status: [] as StatusEntry[],
    statusLoaded: false,
    historyPath: null as string | null,
    settingsOpen: false,
    projectSettingsOpen: false,
    toasts: [] as Toast[],
  }),

  getters: {
    localPath: (s) => s.project?.entry.local_path ?? "",
    isWc: (s) => s.project?.wc != null,
    changeCount: (s) => s.status.length,
    conflictCount: (s) => s.status.filter((e) => e.item === "conflicted").length,
    showPublish: (s) =>
      s.project !== null &&
      (s.project.wporg !== null || s.project.entry.publish !== null || s.project.wc === null),
    /** repo-relative location of the working copy, e.g. "trunk" or "branches/x" */
    currentLocation: (s) => {
      if (!s.layout) return "";
      const rel = s.layout.current_url.slice(s.layout.repo_root.length);
      return rel.replace(/^\//, "") || "/";
    },
  },

  actions: {
    toast(tone: Toast["tone"], text: string) {
      const id = ++toastSeq;
      this.toasts.push({ id, tone, text });
      setTimeout(() => {
        this.toasts = this.toasts.filter((t) => t.id !== id);
      }, tone === "error" ? 7000 : 4000);
    },

    async bootstrap() {
      try {
        [this.svn, this.config] = await Promise.all([api.detectSvn(), api.getConfig()]);
      } catch (e) {
        this.toast("error", errorMessage(e));
      }
      this.svnChecked = true;
    },

    async openProject(localPath: string) {
      this.project = await api.openProject(localPath);
      this.config = await api.getConfig();
      this.layout = null;
      this.status = [];
      this.statusLoaded = false;
      this.historyPath = null;
      this.view = this.project.wc ? "changes" : "publish";
      if (this.project.wc) {
        void this.refreshStatus();
        void this.refreshLayout();
      }
    },

    closeProject() {
      this.project = null;
      this.layout = null;
      this.status = [];
      this.statusLoaded = false;
    },

    async refreshStatus() {
      if (!this.isWc) return;
      try {
        this.status = await api.wcStatus(this.localPath);
        this.statusLoaded = true;
      } catch (e) {
        this.toast("error", errorMessage(e));
      }
    },

    async refreshLayout() {
      if (!this.isWc) return;
      try {
        this.layout = await api.repoLayout(this.localPath);
      } catch {
        this.layout = null; // offline or repo unreachable — hide repo sections
      }
    },

    async reloadProject() {
      if (!this.project) return;
      this.project = await api.openProject(this.project.entry.local_path);
      this.config = await api.getConfig();
    },

    async refreshConfig() {
      this.config = await api.getConfig();
    },
  },
});
