<script setup lang="ts">
import { computed, ref } from "vue";
import { api, errorMessage, isAppError } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import type { DeployPreview } from "@/lib/types";
import DiffPane from "../DiffPane.vue";
import FileTree from "../FileTree.vue";
import UiSplit from "../ui/UiSplit.vue";
import UiEmpty from "../ui/UiEmpty.vue";
import UiIcon from "../ui/UiIcon.vue";
import AuthDialog from "../dialogs/AuthDialog.vue";

const app = useAppStore();

type Phase = "idle" | "preparing" | "ready" | "pushing";
const phase = ref<Phase>("idle");
const preview = ref<DeployPreview | null>(null);
const activeFile = ref<string | null>(null);
const diff = ref("");
const diffLoading = ref(false);
const message = ref("");
const authOpen = ref(false);

const publish = computed(() => app.project?.entry.publish ?? null);
const wporgHost = computed(() =>
  (publish.value?.kind ?? app.project?.wporg?.kind) === "theme"
    ? "themes.svn.wordpress.org"
    : "plugins.svn.wordpress.org"
);
const changeCount = computed(() => preview.value?.status.length ?? 0);

async function prepare() {
  phase.value = "preparing";
  preview.value = null;
  activeFile.value = null;
  try {
    preview.value = await api.publishPrepare(app.localPath);
    phase.value = "ready";
  } catch (e) {
    app.toast("error", errorMessage(e));
    phase.value = "idle";
  }
}

async function showDiff(path: string) {
  activeFile.value = path;
  diffLoading.value = true;
  try {
    const d = await api.publishDiff(app.localPath, path);
    diff.value = d.trim() ? d : "(no text diff — new file, binary, or property change)";
  } catch (e) {
    diff.value = errorMessage(e);
  } finally {
    diffLoading.value = false;
  }
}

async function push(username?: string, password?: string) {
  phase.value = "pushing";
  try {
    const rev = await api.publishPush(app.localPath, message.value, username, password);
    authOpen.value = false;
    app.toast("ok", rev === null ? "Nothing to publish — trunk already matches." : `Published — committed r${rev}.`);
    message.value = "";
    phase.value = "idle";
    preview.value = null;
  } catch (e) {
    if (isAppError(e) && e.kind === "auth_required") {
      authOpen.value = true;
      phase.value = "ready";
    } else {
      app.toast("error", errorMessage(e));
      phase.value = "ready";
    }
  }
}
</script>

<template>
  <div v-if="!publish" class="flex h-full flex-col items-center justify-center gap-3 p-6">
    <p class="max-w-[380px] text-center text-[13px] leading-relaxed text-muted">
      Publishing pushes this project to a WordPress.org plugin or theme repository.
      Configure the wp.org slug in project settings to enable it.
    </p>
    <button class="btn btn-primary" @click="app.projectSettingsOpen = true">
      <UiIcon name="gear" />Configure publishing
    </button>
  </div>

  <div v-else class="flex h-full flex-col">
    <UiSplit storage-key="publish" :initial="340" :min="260" :max="560">
      <template #left>
        <div class="flex h-full min-h-0 flex-col bg-surface">
          <div class="flex items-center justify-between gap-2 border-b border-edge px-3 py-1.5">
            <span class="truncate text-[12px] text-muted">
              {{ phase === "ready" ? `${changeCount} change${changeCount === 1 ? "" : "s"} vs trunk` : `${publish.slug} → ${wporgHost}` }}
            </span>
            <button class="btn" :disabled="phase === 'preparing' || phase === 'pushing'" @click="prepare">
              {{ phase === "preparing" ? "Preparing…" : preview ? "Refresh" : "Review" }}
            </button>
          </div>

          <UiEmpty
            v-if="phase === 'preparing'"
            text="Staging your files against wp.org trunk… the first run checks out the repository and can take a minute."
          />
          <UiEmpty
            v-else-if="preview && preview.status.length === 0"
            text="Trunk already matches your local files. Nothing to publish."
          />
          <UiEmpty
            v-else-if="!preview"
            text="Review stages your files against the wp.org trunk and shows exactly what publishing will change."
          />
          <FileTree
            v-else
            :entries="preview.status"
            :active-file="activeFile"
            @open="showDiff"
          />

          <div class="flex flex-col gap-2 border-t border-edge p-3">
            <input
              class="field"
              v-model="message"
              placeholder='Commit message — e.g. "Release 1.2.0"'
              :disabled="phase === 'pushing'"
            />
            <button
              class="btn btn-primary self-end"
              :disabled="phase !== 'ready' || changeCount === 0 || !message.trim()"
              @click="push(publish.username || undefined)"
            >
              <UiIcon name="upload" />{{ phase === "pushing" ? "Publishing…" : "Publish to trunk" }}
            </button>
          </div>
        </div>
      </template>
      <template #right>
        <DiffPane v-if="activeFile" :diff="diff" :loading="diffLoading" />
        <UiEmpty v-else text="Select a staged file to see its diff." />
      </template>
    </UiSplit>
  </div>

  <AuthDialog
    v-if="authOpen"
    :host="wporgHost"
    :initial-username="publish?.username ?? ''"
    :busy="phase === 'pushing'"
    @cancel="authOpen = false"
    @submit="(u, p) => push(u, p)"
  />
</template>
