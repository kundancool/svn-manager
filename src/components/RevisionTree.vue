<script setup lang="ts">
import { ref, watch } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import type { RemoteEntry } from "@/lib/types";

// Lazy repository tree pinned to a revision — the "File Tree" of a commit.

const props = defineProps<{ revision: number }>();
const app = useAppStore();

interface Node {
  name: string;
  path: string;
  kind: "dir" | "file";
  size: number | null;
  loaded: boolean;
  open: boolean;
  loading: boolean;
  children: Node[];
}

const roots = ref<Node[]>([]);
const loading = ref(true);
const error = ref("");

function toNode(parent: string, e: RemoteEntry): Node {
  return {
    name: e.name,
    path: parent ? `${parent}/${e.name}` : e.name,
    kind: e.kind,
    size: e.size,
    loaded: false,
    open: false,
    loading: false,
    children: [],
  };
}

function sortNodes(nodes: Node[]): Node[] {
  return nodes.sort((a, b) =>
    a.kind === b.kind ? a.name.localeCompare(b.name) : a.kind === "dir" ? -1 : 1
  );
}

async function loadRoot() {
  loading.value = true;
  error.value = "";
  try {
    const entries = await api.repoBrowse(app.localPath, "", props.revision);
    roots.value = sortNodes(entries.map((e) => toNode("", e)));
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

async function toggle(node: Node) {
  if (node.kind !== "dir") return;
  node.open = !node.open;
  if (node.open && !node.loaded) {
    node.loading = true;
    try {
      const entries = await api.repoBrowse(app.localPath, node.path, props.revision);
      node.children = sortNodes(entries.map((e) => toNode(node.path, e)));
      node.loaded = true;
    } catch (e) {
      app.toast("error", errorMessage(e));
      node.open = false;
    } finally {
      node.loading = false;
    }
  }
}

function fmtSize(n: number | null): string {
  if (n === null) return "";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

watch(() => props.revision, loadRoot, { immediate: true });
</script>

<template>
  <p v-if="loading" class="p-4 text-[12.5px] text-faint">Loading tree at r{{ revision }}…</p>
  <p v-else-if="error" class="p-4 text-[12.5px] text-del">{{ error }}</p>
  <ul v-else class="h-full overflow-y-auto p-2">
    <template v-for="node in roots" :key="node.path">
      <RevTreeRow :node="node" :depth="0" :toggle="toggle" :fmt-size="fmtSize" />
    </template>
  </ul>
</template>

<script lang="ts">
import { defineComponent, h, type PropType } from "vue";
import UiIconComp from "./ui/UiIcon.vue";
import { fileIcon as fileIconSpec } from "@/lib/fileIcons";

// Small recursive row renderer (render function keeps it self-contained).
const RevTreeRow = defineComponent({
  name: "RevTreeRow",
  props: {
    node: { type: Object as PropType<any>, required: true },
    depth: { type: Number, required: true },
    toggle: { type: Function as PropType<(n: any) => void>, required: true },
    fmtSize: { type: Function as PropType<(n: number | null) => string>, required: true },
  },
  setup(props) {
    return () => {
      const n = props.node;
      const row = h(
        "li",
        [
          h(
            "button",
            {
              class:
                "flex w-full items-center gap-1.5 rounded-md py-[3px] pr-2 text-left hover:bg-surface-2",
              style: { paddingLeft: `${6 + props.depth * 14}px` },
              onClick: () => props.toggle(n),
            },
            [
              n.kind === "dir"
                ? h(UiIconComp, {
                    name: "chevron-right",
                    size: 11,
                    class: ["transition-transform text-faint", n.open ? "rotate-90" : ""],
                  })
                : h("span", { class: "w-[11px] shrink-0" }),
              h(UiIconComp, {
                name: n.kind === "dir" ? (n.open ? "folder-open" : "folder") : fileIconSpec(n.name).icon,
                size: 13,
                class: n.kind === "dir" ? "text-accent" : fileIconSpec(n.name).class,
              }),
              h(
                "span",
                { class: ["truncate font-mono text-[12px]", n.kind === "dir" ? "font-semibold" : ""] },
                n.loading ? `${n.name}…` : n.name
              ),
              h("span", { class: "ml-auto shrink-0 font-mono text-[10.5px] text-faint" }, props.fmtSize(n.size)),
            ]
          ),
          n.open && n.children.length
            ? h(
                "ul",
                n.children.map((c: any) =>
                  h(RevTreeRow, {
                    key: c.path,
                    node: c,
                    depth: props.depth + 1,
                    toggle: props.toggle,
                    fmtSize: props.fmtSize,
                  })
                )
              )
            : null,
        ]
      );
      return row;
    };
  },
});
export { RevTreeRow };
</script>
