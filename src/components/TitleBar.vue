<script setup lang="ts">
import { useAppStore } from "@/stores/app";
import { api, errorMessage } from "@/lib/api";
import UiIcon from "./ui/UiIcon.vue";
import { ref } from "vue";

const app = useAppStore();
const updating = ref(false);

async function update() {
  if (!app.isWc) return;
  updating.value = true;
  try {
    const rev = await api.wcUpdate(app.localPath);
    app.toast("ok", `Updated to r${rev}.`);
    await Promise.all([app.refreshStatus(), app.reloadProject()]);
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    updating.value = false;
  }
}

async function refresh() {
  await app.refreshStatus();
  await app.refreshLayout();
}

async function cleanup() {
  try {
    await api.wcCleanup(app.localPath);
    app.toast("ok", "Working copy cleaned up.");
  } catch (e) {
    app.toast("error", errorMessage(e));
  }
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-11 shrink-0 items-center gap-2 border-b border-edge bg-chrome pl-[84px] pr-3"
  >
    <div data-tauri-drag-region class="flex min-w-0 items-center gap-2">
      <template v-if="app.project">
        <button class="btn btn-ghost px-1.5 py-1 text-muted" title="All projects" @click="app.closeProject()">
          <UiIcon name="back" />
        </button>
        <span class="truncate text-[13px] font-semibold">{{ app.project.entry.name }}</span>
        <span v-if="app.currentLocation" class="truncate rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[11px] text-muted">
          {{ app.currentLocation }}
        </span>
      </template>
      <span v-else class="text-[13px] font-semibold text-muted">SVN Manager</span>
    </div>

    <div data-tauri-drag-region class="flex-1"></div>

    <template v-if="app.project && app.isWc">
      <button class="btn" :disabled="updating" @click="update">
        <UiIcon name="download" />{{ updating ? "Updating…" : "Update" }}
      </button>
      <button class="btn btn-ghost px-2 text-muted" title="Refresh" @click="refresh">
        <UiIcon name="refresh" />
      </button>
      <button class="btn btn-ghost px-2 text-muted" title="Cleanup working copy (repair after interrupted operations)" @click="cleanup">
        <UiIcon name="wrench" />
      </button>
    </template>
    <button
      v-if="app.project"
      class="btn btn-ghost px-2 text-muted"
      title="Project settings"
      @click="app.projectSettingsOpen = true"
    >
      <UiIcon name="folder" />
    </button>
    <button class="btn btn-ghost px-2 text-muted" title="Settings" @click="app.settingsOpen = true">
      <UiIcon name="gear" />
    </button>
  </header>
</template>
