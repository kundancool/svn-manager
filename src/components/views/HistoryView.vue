<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import { usePrefsStore } from "@/stores/prefs";
import { formatDate } from "@/lib/theme";
import type { LogEntry } from "@/lib/types";
import DiffPane from "../DiffPane.vue";
import RevisionTree from "../RevisionTree.vue";
import UiVSplit from "../ui/UiVSplit.vue";
import UiEmpty from "../ui/UiEmpty.vue";
import UiIcon from "../ui/UiIcon.vue";

const app = useAppStore();
const prefs = usePrefsStore();

const entries = ref<LogEntry[]>([]);
const search = ref("");
const hasMore = ref(false);
const active = ref<LogEntry | null>(null);
const loading = ref(false);

type Tab = "commit" | "changes" | "tree";
const tab = ref<Tab>("changes");

// per-commit changes tab state
const activePath = ref<string | null>(null);
const pathDiff = ref("");
const pathDiffLoading = ref(false);

const actionColor: Record<string, string> = {
  A: "text-ok", M: "text-mod", D: "text-del", R: "text-warn",
};

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return entries.value;
  return entries.value.filter(
    (e) =>
      e.message.toLowerCase().includes(q) ||
      (e.author ?? "").toLowerCase().includes(q) ||
      `r${e.revision}`.includes(q)
  );
});

/** repo-relative location of the WC, e.g. "/trunk"; "" when at root */
const wcPrefix = computed(() => {
  if (!app.layout) return "";
  const rel = app.layout.current_url.slice(app.layout.repo_root.length);
  return rel === "/" ? "" : rel;
});

/** map a log path (repo-absolute, "/trunk/x") to a WC-relative path, or null */
function toWcRelative(repoPath: string): string | null {
  const prefix = wcPrefix.value;
  if (prefix === "") return repoPath.replace(/^\//, "");
  if (repoPath === prefix) return "";
  if (repoPath.startsWith(prefix + "/")) return repoPath.slice(prefix.length + 1);
  return null;
}

function clearFileFilter() {
  app.historyPath = null;
  active.value = null;
  load();
}

async function load(before?: number) {
  loading.value = true;
  try {
    const size = prefs.historyPageSize;
    const page = await api.wcLog(app.localPath, size, before, app.historyPath ?? undefined);
    entries.value = before ? [...entries.value, ...page] : page;
    hasMore.value = page.length === size;
    if (!before && !active.value && page.length > 0) select(page[0]);
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    loading.value = false;
  }
}

function select(entry: LogEntry) {
  active.value = entry;
  activePath.value = null;
  pathDiff.value = "";
  if (tab.value === "tree") tab.value = "changes";
}

async function showPathDiff(repoPath: string) {
  if (!active.value) return;
  activePath.value = repoPath;
  pathDiffLoading.value = true;
  try {
    const rel = toWcRelative(repoPath);
    const d = await api.wcRevisionDiff(app.localPath, active.value.revision, rel ?? undefined);
    pathDiff.value = d.trim()
      ? d
      : rel === null
        ? "(path is outside this working copy — switch to that branch to inspect it)"
        : "(no text diff — added binary, directory, or property change)";
  } catch (e) {
    pathDiff.value = errorMessage(e);
  } finally {
    pathDiffLoading.value = false;
  }
}

async function rollback() {
  if (!active.value) return;
  if (prefs.confirmDestructive) {
    const yes = await ask(
      `Reverse-merge r${active.value.revision} into the working copy? The undo appears as local changes for review.`,
      { title: "Roll back revision", kind: "warning" }
    );
    if (!yes) return;
  }
  try {
    await api.rollbackRevision(app.localPath, active.value.revision);
    app.toast("ok", `r${active.value.revision} reverse-merged. Review the undo in Changes, then commit.`);
    await app.refreshStatus();
    app.view = "changes";
  } catch (e) {
    app.toast("error", errorMessage(e));
  }
}

load();
watch(() => app.historyPath, () => { active.value = null; load(); });
</script>

<template>
  <UiVSplit storage-key="history" :initial="260" :min="140" :max="560">
    <!-- top: commit table -->
    <template #top>
      <div class="flex h-full min-h-0 flex-col bg-surface">
        <div class="flex shrink-0 items-center gap-2 border-b border-edge px-2 py-1.5">
          <div class="relative flex-1">
            <UiIcon name="search" :size="12" class="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-faint" />
            <input class="field py-1 pl-7 text-[12px]" v-model="search" placeholder="Filter by message, author, revision" />
          </div>
          <span
            v-if="app.historyPath"
            class="flex max-w-[40%] items-center gap-1 rounded-full bg-accent/12 py-0.5 pl-2 pr-1 font-mono text-[10.5px] font-semibold text-accent"
          >
            <span class="truncate">{{ app.historyPath }}</span>
            <button class="rounded-full p-0.5 hover:bg-accent/20" aria-label="Clear file filter" @click="clearFileFilter">
              <UiIcon name="x" :size="10" />
            </button>
          </span>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto">
          <UiEmpty v-if="filtered.length === 0 && !loading" :text="search ? 'No revisions match the filter.' : 'No revisions yet.'" />
          <table v-else class="w-full text-[12.5px]">
            <thead>
              <tr class="sticky top-0 z-10 bg-surface text-left">
                <th class="microlabel px-3 py-1.5 font-semibold">Subject</th>
                <th class="microlabel w-32 px-2 py-1.5 font-semibold">Author</th>
                <th class="microlabel w-20 px-2 py-1.5 font-semibold">Revision</th>
                <th class="microlabel hidden w-44 px-3 py-1.5 text-right font-semibold md:table-cell">Date</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="entry in filtered"
                :key="entry.revision"
                class="cursor-default border-t border-edge/50 hover:bg-surface-2"
                :class="active?.revision === entry.revision ? 'bg-surface-2' : ''"
                @click="select(entry)"
              >
                <td class="max-w-0 truncate px-3 py-1.5 font-medium">{{ entry.message || "(no message)" }}</td>
                <td class="truncate px-2 text-muted">{{ entry.author ?? "unknown" }}</td>
                <td class="px-2 font-mono font-semibold text-accent">r{{ entry.revision }}</td>
                <td class="hidden px-3 text-right text-faint md:table-cell">{{ formatDate(entry.date) }}</td>
              </tr>
            </tbody>
          </table>
          <div v-if="hasMore" class="p-2">
            <button class="btn w-full" :disabled="loading" @click="load(entries[entries.length - 1]?.revision)">
              {{ loading ? "Loading…" : "Load older revisions" }}
            </button>
          </div>
        </div>
      </div>
    </template>

    <!-- bottom: commit detail tabs -->
    <template #bottom>
      <UiEmpty v-if="!active" text="Select a commit above to inspect it." />
      <div v-else class="flex h-full min-h-0 flex-col">
        <div class="flex shrink-0 items-center gap-1 border-b border-edge bg-surface px-2 py-1">
          <button
            v-for="t in ([['commit', 'Commit'], ['changes', 'Changes'], ['tree', 'File Tree']] as [Tab, string][])"
            :key="t[0]"
            class="rounded-md px-3 py-1 text-[12px]"
            :class="tab === t[0] ? 'bg-surface-2 font-semibold text-ink' : 'text-muted hover:bg-surface-2/60'"
            @click="tab = t[0]"
          >{{ t[1] }}</button>
          <span class="flex-1"></span>
          <span class="font-mono text-[11.5px] text-accent">r{{ active.revision }}</span>
          <button class="btn btn-ghost px-2 text-muted" title="Reverse-merge this revision into the working copy" @click="rollback">
            <UiIcon name="undo" :size="13" />
          </button>
        </div>

        <!-- Commit tab -->
        <div v-if="tab === 'commit'" class="min-h-0 flex-1 overflow-y-auto p-4">
          <div class="flex flex-col gap-3">
            <div class="flex items-center gap-3">
              <span class="flex h-10 w-10 items-center justify-center rounded-full bg-surface-2 text-[14px] font-bold text-accent">
                {{ (active.author ?? "?").slice(0, 1).toUpperCase() }}
              </span>
              <div>
                <div class="text-[13px] font-semibold">{{ active.author ?? "unknown" }}</div>
                <div class="text-[11.5px] text-faint">{{ formatDate(active.date) }} · r{{ active.revision }} · {{ active.paths.length }} path{{ active.paths.length === 1 ? "" : "s" }}</div>
              </div>
            </div>
            <p class="whitespace-pre-wrap rounded-lg border border-edge bg-surface p-3 text-[13px] leading-relaxed">{{ active.message || "(no message)" }}</p>
          </div>
        </div>

        <!-- Changes tab -->
        <div v-else-if="tab === 'changes'" class="flex min-h-0 flex-1">
          <ul class="w-[300px] shrink-0 overflow-y-auto border-r border-edge bg-surface p-1.5">
            <li v-for="p in active.paths" :key="p.path">
              <button
                class="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left hover:bg-surface-2"
                :class="activePath === p.path ? 'bg-surface-2' : ''"
                @click="showPathDiff(p.path)"
              >
                <span class="w-4 shrink-0 text-center font-mono text-[11px] font-bold" :class="actionColor[p.action] ?? 'text-muted'">{{ p.action }}</span>
                <span class="truncate font-mono text-[12px]">{{ p.path }}</span>
              </button>
            </li>
          </ul>
          <div class="min-h-0 min-w-0 flex-1">
            <DiffPane v-if="activePath" :diff="pathDiff" :loading="pathDiffLoading" />
            <UiEmpty v-else text="Select a changed path to see its diff in this revision." />
          </div>
        </div>

        <!-- File Tree tab -->
        <div v-else class="min-h-0 flex-1">
          <RevisionTree :revision="active.revision" />
        </div>
      </div>
    </template>
  </UiVSplit>
</template>
