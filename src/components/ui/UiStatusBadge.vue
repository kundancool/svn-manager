<script setup lang="ts">
import type { ItemStatus } from "@/lib/types";

defineProps<{ item: ItemStatus }>();

const letters: Partial<Record<ItemStatus, string>> = {
  added: "A", modified: "M", deleted: "D", replaced: "R",
  conflicted: "C", missing: "!", unversioned: "?",
};
const colors: Partial<Record<ItemStatus, string>> = {
  added: "text-ok bg-ok/12",
  modified: "text-mod bg-mod/12",
  deleted: "text-del bg-del/12",
  missing: "text-del bg-del/12",
  replaced: "text-warn bg-warn/12",
  conflicted: "text-warn bg-warn/12",
  unversioned: "text-muted bg-surface-2",
};
</script>

<template>
  <span
    class="inline-flex h-[17px] w-[17px] shrink-0 items-center justify-center rounded font-mono text-[10.5px] font-bold"
    :class="colors[item] ?? 'text-muted bg-surface-2'"
    :title="item"
  >{{ letters[item] ?? "•" }}</span>
</template>
