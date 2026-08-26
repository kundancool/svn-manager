import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  BlameLine,
  CommandLogEntry,
  AppErrorPayload,
  DeployPreview,
  LogEntry,
  OpenedProject,
  PublishConfig,
  RemoteEntry,
  RepoLayout,
  StatusEntry,
  SvnBinary,
} from "./types";

export function isAppError(e: unknown): e is AppErrorPayload {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}

export function errorMessage(e: unknown): string {
  if (isAppError(e)) return e.message;
  return String(e);
}

export const api = {
  // app + settings
  detectSvn: () => invoke<SvnBinary | null>("detect_svn_binary"),
  setSvnPath: (path: string | null) => invoke<SvnBinary | null>("set_svn_path", { path }),
  getConfig: () => invoke<AppConfig>("get_config"),
  saveCredential: (host: string, username: string, password: string) =>
    invoke<AppConfig>("save_credential", { host, username, password }),
  deleteCredential: (host: string) => invoke<AppConfig>("delete_credential", { host }),

  watchStart: (localPath: string) => invoke<void>("watch_start", { localPath }),
  watchStop: () => invoke<void>("watch_stop"),
  getDebugLogs: () => invoke<CommandLogEntry[]>("get_debug_logs"),
  clearDebugLogs: () => invoke<void>("clear_debug_logs"),

  // projects
  openProject: (localPath: string) => invoke<OpenedProject>("open_project", { localPath }),
  forgetProject: (localPath: string) => invoke<AppConfig>("forget_project", { localPath }),
  checkoutProject: (url: string, dest: string) =>
    invoke<OpenedProject>("checkout_project", { url, dest }),
  browseUrl: (url: string, path: string) => invoke<RemoteEntry[]>("browse_url", { url, path }),

  // working copy
  wcStatus: (localPath: string) => invoke<StatusEntry[]>("wc_status", { localPath }),
  wcUpdate: (localPath: string) => invoke<number>("wc_update", { localPath }),
  wcLog: (localPath: string, limit: number, before?: number, path?: string) =>
    invoke<LogEntry[]>("wc_log", { localPath, limit, before: before ?? null, path: path ?? null }),
  wcBlame: (localPath: string, path: string) =>
    invoke<BlameLine[]>("wc_blame", { localPath, path }),
  wcCleanup: (localPath: string) => invoke<void>("wc_cleanup", { localPath }),
  wcLock: (localPath: string, paths: string[], comment?: string) =>
    invoke<void>("wc_lock", { localPath, paths, comment: comment ?? null }),
  wcUnlock: (localPath: string, paths: string[]) =>
    invoke<void>("wc_unlock", { localPath, paths }),
  wcIgnore: (localPath: string, path: string) =>
    invoke<void>("wc_ignore", { localPath, path }),
  wcDiff: (localPath: string, file: string | null) =>
    invoke<string>("wc_diff", { localPath, file }),
  wcRevisionDiff: (localPath: string, revision: number, path?: string) =>
    invoke<string>("wc_revision_diff", { localPath, revision, path: path ?? null }),
  wcCommit: (
    localPath: string,
    message: string,
    paths: string[],
    username?: string,
    password?: string,
  ) =>
    invoke<number | null>("wc_commit", {
      localPath,
      message,
      paths,
      username: username ?? null,
      password: password ?? null,
    }),
  wcRevert: (localPath: string, paths: string[]) =>
    invoke<void>("wc_revert", { localPath, paths }),
  wcResolve: (localPath: string, path: string, accept: string) =>
    invoke<void>("wc_resolve", { localPath, path, accept }),

  // repository
  repoLayout: (localPath: string) => invoke<RepoLayout>("repo_layout", { localPath }),
  repoBrowse: (localPath: string, path: string, revision?: number) =>
    invoke<RemoteEntry[]>("repo_browse", { localPath, path, revision: revision ?? null }),
  switchBranch: (localPath: string, url: string) =>
    invoke<number>("switch_branch", { localPath, url }),
  createCopy: (localPath: string, destination: string, message: string) =>
    invoke<number | null>("create_copy", {
      localPath,
      destination,
      message,
      username: null,
      password: null,
    }),
  mergeUrl: (localPath: string, sourceUrl: string) =>
    invoke<string>("merge_url", { localPath, sourceUrl }),
  rollbackRevision: (localPath: string, revision: number) =>
    invoke<string>("rollback_revision", { localPath, revision }),

  // wp.org publish
  savePublish: (localPath: string, publish: PublishConfig | null) =>
    invoke<AppConfig>("save_publish", { localPath, publish }),
  publishPrepare: (localPath: string) => invoke<DeployPreview>("publish_prepare", { localPath }),
  publishDiff: (localPath: string, file: string | null) =>
    invoke<string>("publish_diff", { localPath, file }),
  publishPush: (localPath: string, message: string, username?: string, password?: string) =>
    invoke<number | null>("publish_push", {
      localPath,
      message,
      username: username ?? null,
      password: password ?? null,
    }),
};
