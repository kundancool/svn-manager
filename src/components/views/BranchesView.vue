<script setup lang="ts">
import { ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import { formatDate } from "@/lib/theme";
import type { RemoteEntry } from "@/lib/types";
import UiIcon from "../ui/UiIcon.vue";
import UiEmpty from "../ui/UiEmpty.vue";
import CopyDialog from "../dialogs/CopyDialog.vue";

const app = useAppStore();

const branches = ref<RemoteEntry[]>([]);
const tags = ref<RemoteEntry[]>([]);
const loading = ref(false);
const busy = ref("");
const copyMode = ref<"branch" | "tag" | null>(null);

function urlFor(kind: "trunk" | "branches" | "tags", name?: string): string {
  const root = app.layout!.repo_root.replace(/\/+$/, "");
  return name ? `${root}/${kind}/${name}` : `${root}/${kind}`;
}

function isCurrent(url: string): boolean {
  return app.layout?.current_url === url;
}

async function load() {
  if (!app.layout) return;
  loading.value = true;
  try {
    const [b, t] = await Promise.all([
      app.layout.has_branches ? api.repoBrowse(app.localPath, "branches") : Promise.resolve([]),
      app.layout.has_tags ? api.repoBrowse(app.localPath, "tags") : Promise.resolve([]),
    ]);
    branches.value = b.filter((e) => e.kind === "dir");
    tags.value = t.filter((e) => e.kind === "dir");
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    loading.value = false;
  }
}

async function switchTo(url: string) {
  busy.value = url;
  try {
    const rev = await api.switchBranch(app.localPath, url);
    app.toast("ok", `Switched — now at r${rev}.`);
    await Promise.all([app.reloadProject(), app.refreshLayout(), app.refreshStatus()]);
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = "";
  }
}

async function mergeFrom(url: string) {
  busy.value = `merge:${url}`;
  try {
    await api.mergeUrl(app.localPath, url);
    app.toast("ok", "Merged into the working copy. Review in Changes, then commit.");
    await app.refreshStatus();
    app.view = "changes";
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = "";
  }
}

load();
</script>

<template>
  <div class="flex h-full min-h-0 flex-col overflow-y-auto">
    <header class="flex items-center gap-2 border-b border-edge bg-surface px-3 py-2">
      <h2 class="mr-auto text-[13.5px] font-semibold">Branches & Tags</h2>
      <button class="btn" @click="copyMode = 'branch'"><UiIcon name="branch" />New branch</button>
      <button class="btn" @click="copyMode = 'tag'"><UiIcon name="tag" />New tag</button>
      <button class="btn btn-ghost px-2 text-muted" title="Refresh" :disabled="loading" @click="load">
        <UiIcon name="refresh" />
      </button>
    </header>

    <div class="flex flex-col gap-5 p-4">
      <section v-if="app.layout?.has_trunk" class="flex flex-col gap-1.5">
        <span class="microlabel">Trunk</span>
        <div class="flex items-center gap-2 rounded-lg border border-edge bg-surface px-3 py-2">
          <UiIcon name="branch" class="text-faint" />
          <span class="flex-1 font-mono text-[12.5px]">trunk</span>
          <span v-if="isCurrent(urlFor('trunk'))" class="rounded-full bg-accent/15 px-2 py-0.5 text-[10.5px] font-semibold text-accent">current</span>
          <template v-else>
            <button class="btn" :disabled="busy !== ''" @click="switchTo(urlFor('trunk'))">Switch</button>
            <button class="btn" :disabled="busy !== ''" title="Merge trunk into the working copy" @click="mergeFrom(urlFor('trunk'))">
              <UiIcon name="merge" />Merge
            </button>
          </template>
        </div>
      </section>

      <section class="flex flex-col gap-1.5">
        <span class="microlabel">Branches</span>
        <UiEmpty v-if="branches.length === 0 && !loading" text="No branches yet. Create one from the current location." />
        <div
          v-for="b in branches"
          :key="b.name"
          class="flex items-center gap-2 rounded-lg border border-edge bg-surface px-3 py-2"
        >
          <UiIcon name="branch" class="text-faint" />
          <div class="min-w-0 flex-1">
            <div class="truncate font-mono text-[12.5px]">{{ b.name }}</div>
            <div class="text-[11px] text-faint">
              r{{ b.revision ?? "?" }} · {{ b.author ?? "unknown" }}{{ b.date ? ` · ${formatDate(b.date)}` : "" }}
            </div>
          </div>
          <span v-if="isCurrent(urlFor('branches', b.name))" class="rounded-full bg-accent/15 px-2 py-0.5 text-[10.5px] font-semibold text-accent">current</span>
          <template v-else>
            <button class="btn" :disabled="busy !== ''" @click="switchTo(urlFor('branches', b.name))">Switch</button>
            <button class="btn" :disabled="busy !== ''" :title="`Merge ${b.name} into the working copy`" @click="mergeFrom(urlFor('branches', b.name))">
              <UiIcon name="merge" />Merge
            </button>
          </template>
        </div>
      </section>

      <section class="flex flex-col gap-1.5">
        <span class="microlabel">Tags</span>
        <UiEmpty v-if="tags.length === 0 && !loading" text="No tags yet." />
        <div
          v-for="t in tags"
          :key="t.name"
          class="flex items-center gap-2 rounded-lg border border-edge bg-surface px-3 py-2"
        >
          <UiIcon name="tag" class="text-faint" />
          <div class="min-w-0 flex-1">
            <div class="truncate font-mono text-[12.5px]">{{ t.name }}</div>
            <div class="text-[11px] text-faint">
              r{{ t.revision ?? "?" }} · {{ t.author ?? "unknown" }}{{ t.date ? ` · ${formatDate(t.date)}` : "" }}
            </div>
          </div>
          <button class="btn" :disabled="busy !== ''" @click="switchTo(urlFor('tags', t.name))">Switch</button>
        </div>
      </section>
    </div>
  </div>

  <CopyDialog v-if="copyMode" :mode="copyMode" @close="copyMode = null" @created="load" />
</template>
