<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { api } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import TitleBar from "@/components/TitleBar.vue";
import SideBar from "@/components/SideBar.vue";
import StatusBar from "@/components/StatusBar.vue";
import UiToasts from "@/components/ui/UiToasts.vue";
import WelcomeView from "@/components/views/WelcomeView.vue";
import ChangesView from "@/components/views/ChangesView.vue";
import HistoryView from "@/components/views/HistoryView.vue";
import BranchesView from "@/components/views/BranchesView.vue";
import BrowserView from "@/components/views/BrowserView.vue";
import PublishView from "@/components/views/PublishView.vue";
import SettingsDialog from "@/components/dialogs/SettingsDialog.vue";
import ProjectSettingsDialog from "@/components/dialogs/ProjectSettingsDialog.vue";

const app = useAppStore();
app.bootstrap();

// Realtime working-copy watching: the backend watches the open project's
// folder and emits wc-fs-change on any edit; status refreshes throttled.
let unlisten: UnlistenFn | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let lastRefresh = 0;

function scheduleRefresh() {
  const MIN_GAP = 1500;
  if (refreshTimer) return;
  const due = Math.max(0, MIN_GAP - (Date.now() - lastRefresh));
  refreshTimer = setTimeout(async () => {
    refreshTimer = null;
    lastRefresh = Date.now();
    await app.refreshStatus();
  }, due);
}

watch(
  () => (app.project?.wc ? app.project.entry.local_path : null),
  async (path) => {
    try {
      if (path) await api.watchStart(path);
      else await api.watchStop();
    } catch {
      /* watching is best-effort; manual refresh still works */
    }
  },
  { immediate: true }
);

onMounted(async () => {
  unlisten = await listen("wc-fs-change", scheduleRefresh);
});
onBeforeUnmount(() => {
  unlisten?.();
  if (refreshTimer) clearTimeout(refreshTimer);
  void api.watchStop();
});
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar />

    <div v-if="app.project" class="flex min-h-0 flex-1">
      <SideBar />
      <main class="min-h-0 min-w-0 flex-1 bg-bg">
        <ChangesView v-if="app.view === 'changes' && app.isWc" :key="`c-${app.localPath}`" />
        <HistoryView v-else-if="app.view === 'history' && app.isWc" :key="`h-${app.localPath}`" />
        <BranchesView v-else-if="app.view === 'branches' && app.isWc" :key="`b-${app.localPath}`" />
        <BrowserView v-else-if="app.view === 'browser' && app.isWc" :key="`r-${app.localPath}`" />
        <PublishView v-else :key="`p-${app.localPath}`" />
      </main>
    </div>
    <WelcomeView v-else />

    <StatusBar />
  </div>

  <SettingsDialog v-if="app.settingsOpen" @close="app.settingsOpen = false" />
  <ProjectSettingsDialog
    v-if="app.projectSettingsOpen && app.project"
    :key="app.localPath"
    @close="app.projectSettingsOpen = false"
  />
  <UiToasts />
</template>
