import type { StatusEntry } from "./types";

export interface TreeNode {
  name: string;
  /** repo/wc-relative path of this node */
  path: string;
  isDir: boolean;
  /** status entry attached to this exact path (dirs can have one too, e.g. an added folder) */
  entry: StatusEntry | null;
  children: TreeNode[];
}

/**
 * Fold flat svn status paths into a directory tree. Directories are created
 * implicitly from path segments; an entry whose own path is a directory
 * (added/deleted folder) attaches to that directory node.
 */
export function buildTree(entries: StatusEntry[]): TreeNode[] {
  const root: TreeNode = { name: "", path: "", isDir: true, entry: null, children: [] };
  const dirs = new Map<string, TreeNode>([["", root]]);

  function dirNode(path: string): TreeNode {
    const existing = dirs.get(path);
    if (existing) return existing;
    const idx = path.lastIndexOf("/");
    const parent = dirNode(idx === -1 ? "" : path.slice(0, idx));
    const node: TreeNode = {
      name: idx === -1 ? path : path.slice(idx + 1),
      path,
      isDir: true,
      entry: null,
      children: [],
    };
    parent.children.push(node);
    dirs.set(path, node);
    return node;
  }

  // Entries with children (or trailing segments) become dirs; leaves are files
  // unless another entry's path nests under them.
  const paths = new Set(entries.map((e) => e.path));
  const isDirPath = (p: string) =>
    [...paths].some((other) => other !== p && other.startsWith(p + "/"));

  for (const entry of entries) {
    const p = entry.path.replace(/\/+$/, "");
    if (isDirPath(p)) {
      dirNode(p).entry = entry;
    } else {
      const idx = p.lastIndexOf("/");
      const parent = dirNode(idx === -1 ? "" : p.slice(0, idx));
      parent.children.push({
        name: idx === -1 ? p : p.slice(idx + 1),
        path: p,
        isDir: false,
        entry,
        children: [],
      });
    }
  }

  sort(root);
  return root.children;
}

function sort(node: TreeNode) {
  node.children.sort((a, b) =>
    a.isDir === b.isDir ? a.name.localeCompare(b.name) : a.isDir ? -1 : 1
  );
  node.children.forEach(sort);
}

/** Every status-entry path at or below this node (what selecting a dir selects). */
export function entryPaths(node: TreeNode): string[] {
  const out: string[] = [];
  if (node.entry) out.push(node.entry.path);
  for (const child of node.children) out.push(...entryPaths(child));
  return out;
}
