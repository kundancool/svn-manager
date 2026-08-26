<script setup lang="ts">
import { computed, ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import { formatDate } from "@/lib/theme";
import type { RemoteEntry } from "@/lib/types";
import UiIcon from "../ui/UiIcon.vue";
import { fileIcon } from "@/lib/fileIcons";
import UiEmpty from "../ui/UiEmpty.vue";

// Remote repository browser — navigates `svn ls` from the repo root.

const app = useAppStore();

const path = ref("");
const entries = ref<RemoteEntry[]>([]);
const loading = ref(false);

const crumbs = computed(() => {
  const parts = path.value.split("/").filter(Boolean);
  return parts.map((name, i) => ({ name, path: parts.slice(0, i + 1).join("/") }));
});

function fmtSize(n: number | null): string {
  if (n === null) return "";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

async function load(p: string) {
  loading.value = true;
  try {
    entries.value = await api.repoBrowse(app.localPath, p);
    path.value = p;
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    loading.value = false;
  }
}

function enter(entry: RemoteEntry) {
  if (entry.kind !== "dir") return;
  load(path.value ? `${path.value}/${entry.name}` : entry.name);
}

load("");
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <header class="flex items-center gap-1 border-b border-edge bg-surface px-3 py-2 text-[12.5px]">
      <button class="btn btn-ghost px-1.5 py-0.5 font-mono" @click="load('')">root</button>
      <template v-for="crumb in crumbs" :key="crumb.path">
        <span class="text-faint">/</span>
        <button class="btn btn-ghost px-1.5 py-0.5 font-mono" @click="load(crumb.path)">{{ crumb.name }}</button>
      </template>
      <span class="flex-1"></span>
      <button class="btn btn-ghost px-2 text-muted" title="Refresh" :disabled="loading" @click="load(path)">
        <UiIcon name="refresh" />
      </button>
    </header>

    <UiEmpty v-if="!loading && entries.length === 0" text="Empty directory." />
    <div v-else class="min-h-0 flex-1 overflow-y-auto">
      <table class="w-full text-[12.5px]">
        <thead>
          <tr class="sticky top-0 bg-surface text-left">
            <th class="microlabel px-3 py-1.5 font-semibold">Name</th>
            <th class="microlabel w-24 px-2 py-1.5 font-semibold">Revision</th>
            <th class="microlabel hidden w-32 px-2 py-1.5 font-semibold sm:table-cell">Author</th>
            <th class="microlabel hidden w-20 px-2 py-1.5 text-right font-semibold sm:table-cell">Size</th>
            <th class="microlabel hidden w-44 px-3 py-1.5 text-right font-semibold lg:table-cell">Changed</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in entries"
            :key="entry.name"
            class="border-t border-edge/50 hover:bg-surface-2"
            :class="entry.kind === 'dir' ? 'cursor-pointer' : ''"
            @dblclick="enter(entry)"
            @click="entry.kind === 'dir' && enter(entry)"
          >
            <td class="flex items-center gap-2 px-3 py-1.5">
              <UiIcon :name="entry.kind === 'dir' ? 'folder' : fileIcon(entry.name).icon" :class="entry.kind === 'dir' ? 'text-accent' : fileIcon(entry.name).class" />
              <span class="truncate font-mono">{{ entry.name }}</span>
            </td>
            <td class="px-2 font-mono text-muted">r{{ entry.revision ?? "?" }}</td>
            <td class="hidden truncate px-2 text-muted sm:table-cell">{{ entry.author ?? "" }}</td>
            <td class="hidden px-2 text-right font-mono text-faint sm:table-cell">{{ fmtSize(entry.size) }}</td>
            <td class="hidden px-3 text-right text-faint lg:table-cell">{{ entry.date ? formatDate(entry.date) : "" }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
