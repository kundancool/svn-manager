<script setup lang="ts">
import { useAppStore, type ViewId } from "@/stores/app";
import UiIcon from "./ui/UiIcon.vue";

const app = useAppStore();

interface NavItem {
  id: ViewId;
  label: string;
  icon: string;
  count?: number;
}

function items(): { section: string; items: NavItem[] }[] {
  const out: { section: string; items: NavItem[] }[] = [];
  if (app.isWc) {
    out.push({
      section: "Workspace",
      items: [
        { id: "changes", label: "Changes", icon: "plus", count: app.changeCount },
        { id: "history", label: "History", icon: "clock" },
      ],
    });
    const repo: NavItem[] = [];
    if (app.layout?.has_branches || app.layout?.has_tags) {
      repo.push({ id: "branches", label: "Branches & Tags", icon: "branch" });
    }
    repo.push({ id: "browser", label: "Repository", icon: "globe" });
    out.push({ section: "Remote", items: repo });
  }
  if (app.showPublish) {
    out.push({
      section: "Deploy",
      items: [{ id: "publish", label: "Publish to wp.org", icon: "upload" }],
    });
  }
  return out;
}
</script>

<template>
  <aside class="vibrancy flex w-12 shrink-0 flex-col gap-1 overflow-y-auto border-r border-edge bg-chrome p-2 md:w-52">
    <template v-for="group in items()" :key="group.section">
      <span class="microlabel mt-2 hidden px-2 first:mt-0 md:block">{{ group.section }}</span>
      <button
        v-for="item in group.items"
        :key="item.id"
        class="flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[12.5px] transition-colors"
        :class="app.view === item.id
          ? 'bg-surface-2 font-semibold text-ink'
          : 'text-muted hover:bg-surface-2/60'"
        :title="item.label"
        @click="app.view = item.id"
      >
        <UiIcon :name="item.icon" />
        <span class="hidden flex-1 truncate md:inline">{{ item.label }}</span>
        <span
          v-if="item.count"
          class="hidden rounded-full bg-accent/15 px-1.5 text-[10.5px] font-semibold text-accent md:inline"
        >{{ item.count }}</span>
      </button>
    </template>
  </aside>
</template>
