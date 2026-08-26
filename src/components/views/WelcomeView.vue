<script setup lang="ts">
import { ref } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import UiIcon from "../ui/UiIcon.vue";
import CheckoutDialog from "../dialogs/CheckoutDialog.vue";
import appIcon from "@/assets/icon.png";

const app = useAppStore();
const busy = ref(false);
const checkoutOpen = ref(false);

const platform = navigator.platform.toLowerCase();
const installHint = platform.includes("mac")
  ? "brew install subversion"
  : platform.includes("win")
    ? "winget install TortoiseSVN  (include command line tools)"
    : "sudo apt install subversion";

async function openProject(path: string) {
  busy.value = true;
  try {
    await app.openProject(path);
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = false;
  }
}

async function pickFolder() {
  const picked = await openDialog({ directory: true, title: "Open a folder" });
  if (typeof picked === "string") await openProject(picked);
}

async function forget(path: string) {
  try {
    app.config = await api.forgetProject(path);
  } catch (e) {
    app.toast("error", errorMessage(e));
  }
}
</script>

<template>
  <main class="flex min-h-0 flex-1">
    <!-- left: identity + actions -->
    <section class="vibrancy flex w-[42%] min-w-[320px] max-w-[460px] flex-col justify-center gap-6 border-r border-edge bg-chrome px-10">
      <div class="flex flex-col gap-3">
        <img :src="appIcon" alt="" class="h-20 w-20 drop-shadow-lg" draggable="false" />
        <div>
          <h1 class="text-[24px] font-bold tracking-tight">SVN Manager</h1>
          <p class="mt-1 text-[13px] text-muted">A fast, open source Subversion client.</p>
        </div>
      </div>

      <div v-if="app.svnChecked && !app.svn" class="flex flex-col gap-2.5">
        <p class="text-[13px] leading-relaxed">
          <strong>Subversion isn't installed.</strong> SVN Manager drives the standard
          <code class="font-mono">svn</code> command line tool.
        </p>
        <code class="rounded-md border border-edge bg-bg px-3 py-2 font-mono text-[12px] text-warn">{{ installHint }}</code>
        <button class="btn self-start" @click="app.bootstrap()">
          <UiIcon name="refresh" />Check again
        </button>
      </div>

      <div v-else class="flex flex-col gap-2">
        <button class="btn btn-primary justify-start px-4 py-2.5 text-[13px]" :disabled="busy || !app.svn" @click="pickFolder">
          <UiIcon name="folder" :size="15" />Open folder…
        </button>
        <button class="btn justify-start px-4 py-2.5 text-[13px]" :disabled="busy || !app.svn" @click="checkoutOpen = true">
          <UiIcon name="download" :size="15" />Check out a repository…
        </button>
      </div>
    </section>

    <!-- right: recent projects fill the page -->
    <section class="flex min-w-0 flex-1 flex-col bg-bg">
      <header class="flex h-10 shrink-0 items-center border-b border-edge px-5">
        <span class="microlabel">Recent projects</span>
      </header>

      <div v-if="!app.config || app.config.projects.length === 0" class="flex flex-1 items-center justify-center">
        <p class="max-w-[300px] text-center text-[12.5px] leading-relaxed text-faint">
          Projects you open or check out appear here.
        </p>
      </div>

      <ul v-else class="min-h-0 flex-1 overflow-y-auto p-3">
        <li v-for="p in app.config.projects" :key="p.local_path" class="group flex items-center gap-1">
          <button
            class="flex min-w-0 flex-1 items-center gap-3 rounded-lg border border-transparent px-3 py-2.5 text-left hover:border-edge hover:bg-surface"
            :disabled="busy"
            @click="openProject(p.local_path)"
          >
            <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-surface-2 text-accent">
              <UiIcon name="folder" :size="16" />
            </span>
            <span class="flex min-w-0 flex-col">
              <span class="truncate text-[13.5px] font-semibold">{{ p.name }}</span>
              <span class="truncate font-mono text-[11px] text-faint">{{ p.local_path }}</span>
            </span>
            <span
              v-if="p.publish"
              class="ml-auto shrink-0 rounded-full bg-accent/12 px-2 py-0.5 text-[10.5px] font-semibold text-accent"
            >wp.org</span>
          </button>
          <button
            class="btn btn-ghost invisible px-2 text-faint group-hover:visible"
            :aria-label="`Remove ${p.name} from recents`"
            title="Remove from recents"
            @click="forget(p.local_path)"
          >
            <UiIcon name="x" :size="12" />
          </button>
        </li>
      </ul>
    </section>
  </main>

  <CheckoutDialog v-if="checkoutOpen" @close="checkoutOpen = false" />
</template>
