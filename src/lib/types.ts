export type ItemStatus =
  | "added"
  | "conflicted"
  | "deleted"
  | "external"
  | "ignored"
  | "incomplete"
  | "merged"
  | "missing"
  | "modified"
  | "none"
  | "normal"
  | "obstructed"
  | "replaced"
  | "unversioned"
  | "unknown";

export interface StatusEntry {
  path: string;
  item: ItemStatus;
  props: ItemStatus;
  revision: number | null;
  last_author: string | null;
  last_commit_revision: number | null;
  last_commit_date: string | null;
  has_lock: boolean;
}

export interface BlameLine {
  line_number: number;
  revision: number | null;
  author: string | null;
  date: string | null;
  text: string;
}

export interface LogPath {
  path: string;
  action: string;
  kind: string;
}

export interface LogEntry {
  revision: number;
  author: string | null;
  date: string;
  message: string;
  paths: LogPath[];
}

export interface WcInfo {
  url: string;
  repo_root: string;
  revision: number;
  kind: string;
  wc_root: string | null;
  relative_url: string | null;
}

export interface RemoteEntry {
  name: string;
  kind: "dir" | "file";
  size: number | null;
  revision: number | null;
  author: string | null;
  date: string | null;
}

export interface RepoLayout {
  repo_root: string;
  current_url: string;
  has_trunk: boolean;
  has_branches: boolean;
  has_tags: boolean;
}

export interface SyncReport {
  copied: string[];
  deleted: string[];
}

export interface DeployPreview {
  sync: SyncReport;
  status: StatusEntry[];
}

export interface SvnBinary {
  path: string;
  version: string;
}

export type RepoKind = "plugin" | "theme";

export interface PublishConfig {
  slug: string;
  kind: RepoKind;
  username: string;
  excludes: string[];
}

export interface ProjectEntry {
  local_path: string;
  name: string;
  publish: PublishConfig | null;
}

export interface CredentialMeta {
  host: string;
  username: string;
}

export interface AppConfig {
  svn_path: string | null;
  projects: ProjectEntry[];
  credentials: CredentialMeta[];
}

export interface WporgDetection {
  slug: string;
  kind: RepoKind;
}

export interface OpenedProject {
  entry: ProjectEntry;
  wc: WcInfo | null;
  wporg: WporgDetection | null;
}

export interface CommandLogEntry {
  at: number;
  command: string;
  cwd: string | null;
  had_stdin: boolean;
  exit_code: number | null;
  duration_ms: number;
  stdout: string;
  stderr: string;
  ok: boolean;
}

export interface AppErrorPayload {
  kind:
    | "svn_not_found"
    | "not_a_working_copy"
    | "auth_required"
    | "svn_command"
    | "parse"
    | "io"
    | "config";
  message: string;
}
