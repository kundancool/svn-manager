<script setup lang="ts">
import { computed, ref } from "vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { api, errorMessage, isAppError } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import { usePrefsStore } from "@/stores/prefs";
import { host_of } from "@/lib/host";
import DiffPane from "../DiffPane.vue";
import FileTree from "../FileTree.vue";
import UiSplit from "../ui/UiSplit.vue";
import UiEmpty from "../ui/UiEmpty.vue";
import UiIcon from "../ui/UiIcon.vue";
import AuthDialog from "../dialogs/AuthDialog.vue";
import BlameDialog from "../dialogs/BlameDialog.vue";
import UiContextMenu, { type MenuItem } from "../ui/UiContextMenu.vue";
import type { TreeNode } from "@/lib/tree";

const app = useAppStore();
const prefs = usePrefsStore();

const visibleEntries = computed(() =>
  prefs.showUnversioned ? app.status : app.status.filter((e) => e.item !== "unversioned")
);

// SVN has no index — "staged" is the set of files the next commit includes.
const staged = ref<Set<string>>(new Set());
const unstagedEntries = computed(() => visibleEntries.value.filter((e) => !staged.value.has(e.path)));
const stagedEntries = computed(() => visibleEntries.value.filter((e) => staged.value.has(e.path)));

const activeFile = ref<string | null>(null);
const diff = ref("");
const diffLoading = ref(false);
const message = ref("");
const busy = ref<"" | "commit" | "revert" | "resolve">("");
const authOpen = ref(false);
const authError = ref("");
const menu = ref<{ node: TreeNode; x: number; y: number } | null>(null);
const blamePath = ref<string | null>(null);

const authHost = computed(() => host_of(app.layout?.repo_root ?? app.project?.wc?.url ?? ""));

function setStaged(paths: string[], on: boolean) {
  const valid = new Set(visibleEntries.value.map((e) => e.path));
  const next = new Set(staged.value);
  for (const p of paths) {
    if (!valid.has(p)) continue;
    if (on) next.add(p);
    else next.delete(p);
  }
  staged.value = next;
}

function stageAll() {
  staged.value = new Set(visibleEntries.value.map((e) => e.path));
}
function unstageAll() {
  staged.value = new Set();
}

async function showDiff(path: string) {
  activeFile.value = path;
  diffLoading.value = true;
  try {
    const d = await api.wcDiff(app.localPath, path);
    diff.value = d.trim() ? d : "(no text diff — new file, binary, or property change)";
  } catch (e) {
    diff.value = errorMessage(e);
  } finally {
    diffLoading.value = false;
  }
}

async function afterMutation() {
  staged.value = new Set();
  activeFile.value = null;
  diff.value = "";
  await app.refreshStatus();
}

async function commit(username?: string, password?: string) {
  busy.value = "commit";
  try {
    const rev = await api.wcCommit(app.localPath, message.value, [...staged.value], username, password);
    authOpen.value = false;
    authError.value = "";
    app.toast("ok", rev === null ? "Nothing to commit." : `Committed r${rev}.`);
    message.value = "";
    await afterMutation();
    await app.reloadProject();
  } catch (e) {
    if (isAppError(e) && e.kind === "auth_required") {
      authError.value = username
        ? "Authentication failed — the server rejected these credentials."
        : "";
      authOpen.value = true;
    } else {
      app.toast("error", errorMessage(e));
    }
  } finally {
    busy.value = "";
  }
}

async function resolve(path: string, accept: "working" | "mine-full" | "theirs-full") {
  busy.value = "resolve";
  try {
    await api.wcResolve(app.localPath, path, accept);
    app.toast("ok", `Resolved ${path}.`);
    await app.refreshStatus();
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = "";
  }
}

// ---- context menu ----

function openMenu(node: TreeNode, x: number, y: number) {
  menu.value = { node, x, y };
}

function menuItems(node: TreeNode): MenuItem[] {
  const e = node.entry;
  const items: MenuItem[] = [];
  if (!node.isDir) {
    items.push({ id: "diff", label: "Show diff", icon: "file" });
    if (e?.item !== "unversioned" && e?.item !== "added") {
      items.push({ id: "blame", label: "Blame", icon: "clock" });
      items.push({ id: "history", label: "File history", icon: "file-clock" });
      if (e?.has_lock) items.push({ id: "unlock", label: "Release lock", icon: "lock-open" });
      else items.push({ id: "lock", label: "Lock file", icon: "lock" });
    }
    if (e?.item === "unversioned") {
      items.push({ id: "ignore", label: "Add to svn:ignore", icon: "ban" });
    }
  }
  if (e && e.item !== "unversioned") {
    items.push({ id: "revert", label: node.isDir ? "Revert folder" : "Revert changes", icon: "undo", danger: true });
  }
  return items;
}

async function onMenuPick(id: string) {
  const node = menu.value!.node;
  menu.value = null;
  const path = node.path;
  try {
    switch (id) {
      case "diff":
        await showDiff(path);
        break;
      case "blame":
        blamePath.value = path;
        break;
      case "history":
        app.historyPath = path;
        app.view = "history";
        break;
      case "lock":
        await api.wcLock(app.localPath, [path]);
        app.toast("ok", `Locked ${path}.`);
        await app.refreshStatus();
        break;
      case "unlock":
        await api.wcUnlock(app.localPath, [path]);
        app.toast("ok", `Released lock on ${path}.`);
        await app.refreshStatus();
        break;
      case "ignore":
        await api.wcIgnore(app.localPath, path);
        app.toast("ok", `${path} added to svn:ignore.`);
        await app.refreshStatus();
        break;
      case "revert": {
        if (prefs.confirmDestructive) {
          const yes = await ask(`Discard local changes to ${path}? This cannot be undone.`, {
            title: "Revert changes",
            kind: "warning",
          });
          if (!yes) return;
        }
        await api.wcRevert(app.localPath, [path]);
        app.toast("ok", `Reverted ${path}.`);
        await afterMutation();
        break;
      }
    }
  } catch (e) {
    app.toast("error", errorMessage(e));
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <UiSplit storage-key="changes" :initial="360" :min="280" :max="600">
      <template #left>
        <div class="flex h-full min-h-0 flex-col bg-surface">
          <!-- unstaged -->
          <div class="flex min-h-0 flex-1 flex-col">
            <div class="flex shrink-0 items-center justify-between border-b border-edge px-3 py-1.5">
              <span class="microlabel">Unstaged ({{ unstagedEntries.length }})</span>
              <button
                class="btn btn-ghost px-2 py-0.5 text-[11.5px] text-ok"
                :disabled="unstagedEntries.length === 0 || busy !== ''"
                @click="stageAll"
              ><UiIcon name="plus" :size="11" />Stage all</button>
            </div>
            <UiEmpty
              v-if="app.statusLoaded && visibleEntries.length === 0"
              text="Working copy is clean. Edit files, then Refresh."
            />
            <p v-else-if="unstagedEntries.length === 0" class="p-3 text-[12px] text-faint">Everything staged.</p>
            <FileTree
              v-else
              :entries="unstagedEntries"
              :active-file="activeFile"
              mode="stage"
              :show-resolve="true"
              :busy="busy !== ''"
              @select="setStaged"
              @open="showDiff"
              @resolve="resolve"
              @menu="openMenu"
            />
          </div>

          <!-- staged -->
          <div class="flex min-h-0 flex-1 flex-col border-t border-edge">
            <div class="flex shrink-0 items-center justify-between border-b border-edge px-3 py-1.5">
              <span class="microlabel">Staged ({{ stagedEntries.length }})</span>
              <button
                class="btn btn-ghost px-2 py-0.5 text-[11.5px] text-del"
                :disabled="stagedEntries.length === 0 || busy !== ''"
                @click="unstageAll"
              ><UiIcon name="minus" :size="11" />Unstage all</button>
            </div>
            <p v-if="stagedEntries.length === 0" class="p-3 text-[12px] text-faint">
              Stage files to include them in the commit.
            </p>
            <FileTree
              v-else
              :entries="stagedEntries"
              :active-file="activeFile"
              mode="unstage"
              :busy="busy !== ''"
              @select="setStaged"
              @open="showDiff"
              @menu="openMenu"
            />
          </div>

        </div>
      </template>
      <template #right>
        <div class="flex h-full min-h-0 flex-col">
          <div class="min-h-0 flex-1">
            <DiffPane v-if="activeFile" :diff="diff" :loading="diffLoading" />
            <UiEmpty v-else text="Select a changed file to see its diff." />
          </div>

          <!-- commit box -->
          <div class="flex shrink-0 flex-col gap-2 border-t border-edge bg-surface p-3">
            <textarea
              class="field min-h-14 resize-y"
              v-model="message"
              placeholder="Commit message"
              :disabled="busy === 'commit'"
            ></textarea>
            <div class="flex items-center justify-between gap-2">
              <span v-if="app.conflictCount > 0" class="text-[11.5px] font-semibold text-warn">
                Resolve conflicts before committing
              </span>
              <span v-else class="text-[11.5px] text-faint">
                {{ stagedEntries.length }} file{{ stagedEntries.length === 1 ? "" : "s" }} staged
              </span>
              <button
                class="btn btn-primary"
                :disabled="busy !== '' || stagedEntries.length === 0 || !message.trim() || app.conflictCount > 0"
                @click="commit()"
              >
                {{ busy === "commit" ? "Committing…" : "Commit" }}
              </button>
            </div>
          </div>
        </div>
      </template>
    </UiSplit>
  </div>

  <UiContextMenu
    v-if="menu"
    :x="menu.x"
    :y="menu.y"
    :items="menuItems(menu.node)"
    @pick="onMenuPick"
    @close="menu = null"
  />
  <BlameDialog v-if="blamePath" :path="blamePath" @close="blamePath = null" />

  <AuthDialog
    v-if="authOpen"
    :host="authHost"
    :busy="busy === 'commit'"
    :error="authError"
    @cancel="authOpen = false; authError = ''"
    @submit="(u, p) => commit(u, p)"
  />
</template>
