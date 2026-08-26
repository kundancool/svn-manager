<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { buildTree, type TreeNode } from "@/lib/tree";
import type { StatusEntry } from "@/lib/types";
import FileTreeNode from "./FileTreeNode.vue";

// Directory tree over flat svn status entries: expandable folders, per-file
// badges, optional tri-state selection at every level.

const props = defineProps<{
  entries: StatusEntry[];
  selected?: Set<string>;
  activeFile?: string | null;
  mode?: "plain" | "checkbox" | "stage" | "unstage";
  showResolve?: boolean;
  busy?: boolean;
}>();

const emit = defineEmits<{
  select: [paths: string[], on: boolean];
  open: [path: string];
  resolve: [path: string, accept: "working" | "mine-full" | "theirs-full"];
  menu: [node: TreeNode, x: number, y: number];
}>();

const tree = computed(() => buildTree(props.entries));

// all dirs start expanded; new dirs appearing after a refresh expand too
const expanded = ref<Set<string>>(new Set());
const collapsed = ref<Set<string>>(new Set());

watch(
  tree,
  () => {
    const dirs = new Set<string>();
    const walk = (nodes: typeof tree.value) => {
      for (const n of nodes) {
        if (n.isDir) {
          dirs.add(n.path);
          walk(n.children);
        }
      }
    };
    walk(tree.value);
    expanded.value = new Set([...dirs].filter((d) => !collapsed.value.has(d)));
  },
  { immediate: true }
);

function toggleExpand(path: string) {
  if (expanded.value.has(path)) {
    expanded.value.delete(path);
    collapsed.value.add(path);
  } else {
    expanded.value.add(path);
    collapsed.value.delete(path);
  }
  expanded.value = new Set(expanded.value);
}
</script>

<template>
  <ul class="min-h-0 flex-1 overflow-y-auto p-1.5">
    <FileTreeNode
      v-for="node in tree"
      :key="node.path"
      :node="node"
      :depth="0"
      :selected="selected"
      :active-file="activeFile ?? null"
      :expanded="expanded"
      :mode="mode ?? 'plain'"
      :show-resolve="showResolve ?? false"
      :busy="busy ?? false"
      @toggle-expand="toggleExpand"
      @select="(p, on) => emit('select', p, on)"
      @open="(p) => emit('open', p)"
      @resolve="(p, a) => emit('resolve', p, a)"
      @menu="(n, x, y) => emit('menu', n, x, y)"
    />
  </ul>
</template>
