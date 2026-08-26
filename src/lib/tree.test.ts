import { describe, expect, it } from "vitest";
import { buildTree, entryPaths } from "./tree";
import type { StatusEntry } from "./types";

function entry(path: string, item: StatusEntry["item"] = "modified"): StatusEntry {
  return {
    path,
    item,
    props: "none",
    revision: 1,
    last_author: null,
    last_commit_revision: null,
    last_commit_date: null,
    has_lock: false,
  };
}

describe("buildTree", () => {
  it("nests files under implicit directories, dirs first, sorted", () => {
    const tree = buildTree([
      entry("src/b.ts"),
      entry("readme.md"),
      entry("src/a.ts"),
      entry("assets/img/logo.png", "added"),
    ]);

    expect(tree.map((n) => n.name)).toEqual(["assets", "src", "readme.md"]);
    const src = tree[1];
    expect(src.isDir).toBe(true);
    expect(src.children.map((n) => n.name)).toEqual(["a.ts", "b.ts"]);
    const img = tree[0].children[0];
    expect(img.isDir).toBe(true);
    expect(img.children[0].name).toBe("logo.png");
    expect(img.children[0].entry?.item).toBe("added");
  });

  it("attaches a status entry to a directory that has nested entries", () => {
    const tree = buildTree([entry("newdir", "added"), entry("newdir/file.txt", "added")]);
    expect(tree).toHaveLength(1);
    const dir = tree[0];
    expect(dir.isDir).toBe(true);
    expect(dir.entry?.item).toBe("added");
    expect(dir.children[0].name).toBe("file.txt");
  });

  it("entryPaths collects the dir's own entry plus all descendants", () => {
    const tree = buildTree([entry("d", "added"), entry("d/x.txt"), entry("d/sub/y.txt")]);
    expect(entryPaths(tree[0]).sort()).toEqual(["d", "d/sub/y.txt", "d/x.txt"]);
  });

  it("handles a flat single file", () => {
    const tree = buildTree([entry("single.txt", "unversioned")]);
    expect(tree).toHaveLength(1);
    expect(tree[0].isDir).toBe(false);
    expect(tree[0].path).toBe("single.txt");
  });
});
