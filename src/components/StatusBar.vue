<script setup lang="ts">
import { useAppStore } from "@/stores/app";

const app = useAppStore();
</script>

<template>
  <footer class="flex h-6 shrink-0 items-center gap-3 border-t border-edge bg-chrome px-3 text-[11px] text-faint">
    <template v-if="app.project?.wc">
      <span class="font-mono">r{{ app.project.wc.revision }}</span>
      <span class="hidden truncate font-mono sm:inline">{{ app.project.wc.url }}</span>
      <span class="flex-1"></span>
      <span v-if="app.conflictCount > 0" class="font-semibold text-warn">
        {{ app.conflictCount }} conflict{{ app.conflictCount === 1 ? "" : "s" }}
      </span>
      <span v-if="app.statusLoaded">{{ app.changeCount }} change{{ app.changeCount === 1 ? "" : "s" }}</span>
    </template>
    <span v-else class="flex-1"></span>
    <span v-if="app.svn">svn {{ app.svn.version }}</span>
  </footer>
</template>
