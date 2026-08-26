<script setup lang="ts">
import { computed } from "vue";
import { entryPaths, type TreeNode } from "@/lib/tree";
import { fileIcon } from "@/lib/fileIcons";
import UiIcon from "./ui/UiIcon.vue";
import UiStatusBadge from "./ui/UiStatusBadge.vue";

const props = defineProps<{
  node: TreeNode;
  depth: number;
  selected?: Set<string>;
  activeFile: string | null;
  expanded: Set<string>;
  mode: "plain" | "checkbox" | "stage" | "unstage";
  showResolve: boolean;
  busy: boolean;
}>();

const emit = defineEmits<{
  toggleExpand: [path: string];
  select: [paths: string[], on: boolean];
  open: [path: string];
  resolve: [path: string, accept: "working" | "mine-full" | "theirs-full"];
  menu: [node: TreeNode, x: number, y: number];
}>();

const isOpen = computed(() => props.expanded.has(props.node.path));
const descendants = computed(() => entryPaths(props.node));
const checkedCount = computed(
  () => descendants.value.filter((p) => props.selected?.has(p)).length
);
const allChecked = computed(
  () => descendants.value.length > 0 && checkedCount.value === descendants.value.length
);

function onCheck() {
  emit("select", descendants.value, !allChecked.value);
}
</script>

<template>
  <li>
    <div
      class="group flex items-center gap-1.5 rounded-md py-[3px] pr-2 hover:bg-surface-2"
      :class="!node.isDir && activeFile === node.path ? 'bg-surface-2' : ''"
      :style="{ paddingLeft: `${6 + depth * 14}px` }"
      @contextmenu.prevent="emit('menu', node, $event.clientX, $event.clientY)"
    >
      <!-- disclosure -->
      <button
        v-if="node.isDir"
        class="flex h-4 w-4 shrink-0 items-center justify-center text-faint"
        :aria-label="isOpen ? `Collapse ${node.name}` : `Expand ${node.name}`"
        :aria-expanded="isOpen"
        @click="emit('toggleExpand', node.path)"
      >
        <UiIcon
          name="chevron-right"
          :size="11"
          class="transition-transform"
          :class="isOpen ? 'rotate-90' : ''"
        />
      </button>
      <span v-else class="w-4 shrink-0"></span>

      <input
        v-if="mode === 'checkbox'"
        type="checkbox"
        :checked="allChecked"
        :indeterminate="checkedCount > 0 && !allChecked"
        :aria-label="`Select ${node.path}`"
        @change="onCheck"
      />
      <button
        v-else-if="mode === 'stage'"
        class="invisible flex h-4 w-4 shrink-0 items-center justify-center rounded text-ok hover:bg-ok/15 group-hover:visible"
        :aria-label="`Stage ${node.path}`"
        title="Stage"
        @click="emit('select', descendants, true)"
      >
        <UiIcon name="plus" :size="12" />
      </button>
      <button
        v-else-if="mode === 'unstage'"
        class="invisible flex h-4 w-4 shrink-0 items-center justify-center rounded text-del hover:bg-del/15 group-hover:visible"
        :aria-label="`Unstage ${node.path}`"
        title="Unstage"
        @click="emit('select', descendants, false)"
      >
        <UiIcon name="minus" :size="12" />
      </button>

      <button
        class="flex min-w-0 flex-1 items-center gap-1.5 text-left"
        @click="node.isDir ? emit('toggleExpand', node.path) : emit('open', node.path)"
      >
        <UiIcon
          :name="node.isDir ? (isOpen ? 'folder-open' : 'folder') : fileIcon(node.name).icon"
          :size="13"
          :class="node.isDir ? 'text-accent' : fileIcon(node.name).class"
        />
        <span class="truncate font-mono text-[12px]" :class="node.isDir ? 'font-semibold' : ''">
          {{ node.name }}
        </span>
        <span v-if="node.isDir && !isOpen" class="text-[10.5px] text-faint">{{ descendants.length }}</span>
      </button>

      <UiIcon v-if="node.entry?.has_lock" name="lock" :size="11" class="text-warn" />
      <UiStatusBadge v-if="node.entry" :item="node.entry.item" />
    </div>

    <!-- conflict quick actions -->
    <div
      v-if="showResolve && node.entry?.item === 'conflicted'"
      class="flex gap-1.5 pb-1 pt-0.5"
      :style="{ paddingLeft: `${28 + depth * 14}px` }"
    >
      <button class="btn px-2 py-0.5 text-[11px]" :disabled="busy" @click="emit('resolve', node.path, 'mine-full')">Keep mine</button>
      <button class="btn px-2 py-0.5 text-[11px]" :disabled="busy" @click="emit('resolve', node.path, 'theirs-full')">Take theirs</button>
      <button class="btn px-2 py-0.5 text-[11px]" :disabled="busy" @click="emit('resolve', node.path, 'working')">Mark resolved</button>
    </div>

    <ul v-if="node.isDir && isOpen">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :selected="selected"
        :active-file="activeFile"
        :expanded="expanded"
        :mode="mode"
        :show-resolve="showResolve"
        :busy="busy"
        @toggle-expand="(p) => emit('toggleExpand', p)"
        @menu="(n, x, y) => emit('menu', n, x, y)"
        @select="(p, on) => emit('select', p, on)"
        @open="(p) => emit('open', p)"
        @resolve="(p, a) => emit('resolve', p, a)"
      />
    </ul>
  </li>
</template>
