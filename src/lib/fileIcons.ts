// Extension → icon name (UiIcon map) + semantic color class.
// Everything unknown falls back to the plain file icon.

export interface FileIconSpec {
  icon: string;
  class: string;
}

const byExt: Record<string, FileIconSpec> = {};

function reg(exts: string[], icon: string, cls: string) {
  for (const e of exts) byExt[e] = { icon, class: cls };
}

reg(["js", "mjs", "cjs", "jsx", "ts", "tsx", "vue", "svelte", "py", "rb", "php", "java", "c", "cc", "cpp", "h", "hpp", "rs", "go", "swift", "kt", "cs", "sh", "bash", "zsh", "pl", "lua", "sql"], "file-code", "text-mod");
reg(["css", "scss", "sass", "less", "styl"], "file-code", "text-accent");
reg(["html", "htm", "twig", "blade"], "file-code", "text-warn");
reg(["json", "json5", "jsonc"], "file-json", "text-warn");
reg(["yml", "yaml", "toml", "ini", "conf", "env", "properties", "editorconfig"], "file-cog", "text-muted");
reg(["xml", "plist", "svg"], "file-code", "text-warn");
reg(["md", "mdx", "txt", "rst", "log", "license", "readme"], "file-text", "text-faint");
reg(["png", "jpg", "jpeg", "gif", "webp", "ico", "bmp", "tiff", "avif", "heic"], "file-image", "text-ok");
reg(["zip", "tar", "gz", "tgz", "bz2", "xz", "7z", "rar", "jar", "war"], "file-archive", "text-warn");
reg(["mp4", "mov", "avi", "mkv", "webm"], "file-video", "text-del");
reg(["mp3", "wav", "ogg", "flac", "m4a"], "file-audio", "text-del");
reg(["pdf"], "file-text", "text-del");
reg(["lock"], "file-cog", "text-faint");

const byName: Record<string, FileIconSpec> = {
  "package.json": { icon: "file-json", class: "text-ok" },
  "composer.json": { icon: "file-json", class: "text-ok" },
  "cargo.toml": { icon: "file-cog", class: "text-warn" },
  makefile: { icon: "file-cog", class: "text-muted" },
  dockerfile: { icon: "file-cog", class: "text-mod" },
  ".gitignore": { icon: "file-cog", class: "text-faint" },
};

const fallback: FileIconSpec = { icon: "file", class: "text-faint" };

export function fileIcon(name: string): FileIconSpec {
  const lower = name.toLowerCase();
  if (byName[lower]) return byName[lower];
  const dot = lower.lastIndexOf(".");
  if (dot === -1) return fallback;
  return byExt[lower.slice(dot + 1)] ?? fallback;
}
