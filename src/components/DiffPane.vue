<script setup lang="ts">
import { computed } from "vue";
import { usePrefsStore, type DiffTheme } from "@/stores/prefs";
import UiIcon from "./ui/UiIcon.vue";

const props = defineProps<{ diff: string; loading?: boolean }>();
const prefs = usePrefsStore();

type Kind = "add" | "del" | "hunk" | "meta" | "ctx";

interface Palette {
  bg: string;
  add: string;
  addBg: string;
  del: string;
  delBg: string;
  hunk: string;
  meta: string;
  ctx: string;
}

// "app" follows the app's semantic tokens; the rest are fixed editor themes.
const palettes: Record<Exclude<DiffTheme, "app">, Palette> = {
  monokai: {
    bg: "#272822",
    add: "#a6e22e", addBg: "rgba(166, 226, 46, 0.10)",
    del: "#f92672", delBg: "rgba(249, 38, 114, 0.10)",
    hunk: "#66d9ef", meta: "#75715e", ctx: "#d6d6cb",
  },
  github: {
    bg: "#ffffff",
    add: "#1a7f37", addBg: "#dafbe1",
    del: "#cf222e", delBg: "#ffebe9",
    hunk: "#0969da", meta: "#656d76", ctx: "#1f2328",
  },
  dracula: {
    bg: "#282a36",
    add: "#50fa7b", addBg: "rgba(80, 250, 123, 0.09)",
    del: "#ff5555", delBg: "rgba(255, 85, 85, 0.10)",
    hunk: "#8be9fd", meta: "#6272a4", ctx: "#f8f8f2",
  },
  solarized: {
    bg: "#002b36",
    add: "#859900", addBg: "rgba(133, 153, 0, 0.12)",
    del: "#dc322f", delBg: "rgba(220, 50, 47, 0.12)",
    hunk: "#268bd2", meta: "#586e75", ctx: "#93a1a1",
  },
};

const style = computed(() => {
  const base = { fontSize: `${prefs.diffFontPx}px` };
  if (prefs.diffTheme === "app") {
    return {
      ...base,
      "--d-bg": "color-mix(in srgb, var(--bg) 60%, transparent)",
      "--d-add": "var(--ok)", "--d-add-bg": "var(--diff-add-bg)",
      "--d-del": "var(--del)", "--d-del-bg": "var(--diff-del-bg)",
      "--d-hunk": "var(--mod)", "--d-meta": "var(--faint)", "--d-ctx": "var(--muted)",
    };
  }
  const p = palettes[prefs.diffTheme];
  return {
    ...base,
    "--d-bg": p.bg,
    "--d-add": p.add, "--d-add-bg": p.addBg,
    "--d-del": p.del, "--d-del-bg": p.delBg,
    "--d-hunk": p.hunk, "--d-meta": p.meta, "--d-ctx": p.ctx,
  };
});

function classify(text: string): Kind {
  if (text.startsWith("+++") || text.startsWith("---") || text.startsWith("diff ") || text.startsWith("Index:") || text.startsWith("===")) return "meta";
  if (text.startsWith("@@")) return "hunk";
  if (text.startsWith("+")) return "add";
  if (text.startsWith("-")) return "del";
  return "ctx";
}

const lines = computed(() => props.diff.split("\n").map((text) => ({ kind: classify(text), text })));
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex h-8 shrink-0 items-center justify-end gap-1 border-b border-edge bg-surface px-2">
      <button
        class="btn btn-ghost px-1.5 py-0.5"
        :class="prefs.diffWrap ? 'text-accent' : 'text-faint'"
        :title="prefs.diffWrap ? 'Disable line wrapping' : 'Wrap long lines'"
        :aria-pressed="prefs.diffWrap"
        @click="prefs.set('diffWrap', !prefs.diffWrap)"
      >
        <UiIcon name="wrap-text" :size="13" />
      </button>
    </div>

    <p v-if="loading" class="p-4 text-[12.5px] text-faint">Loading diff…</p>
    <pre
      v-else
      class="diff-root min-h-0 flex-1 overflow-auto p-3 font-mono leading-normal"
      :class="prefs.diffWrap ? 'wrap' : ''"
      :style="style"
    ><span
      v-for="(line, i) in lines"
      :key="i"
      class="line"
      :class="`line-${line.kind}`"
    >{{ line.text || " " }}
</span></pre>
  </div>
</template>

<style scoped>
.diff-root { background: var(--d-bg); }

/* Lines are blocks sized to the widest content so add/del backgrounds
   stay painted across the full horizontal scroll range. */
.line {
  display: block;
  white-space: pre;
  width: max-content;
  min-width: 100%;
  padding-right: 12px;
}
.wrap .line {
  white-space: pre-wrap;
  word-break: break-all;
  width: auto;
  min-width: 0;
}

.line-add { color: var(--d-add); background: var(--d-add-bg); }
.line-del { color: var(--d-del); background: var(--d-del-bg); }
.line-hunk { color: var(--d-hunk); }
.line-meta { color: var(--d-meta); }
.line-ctx { color: var(--d-ctx); }
</style>
