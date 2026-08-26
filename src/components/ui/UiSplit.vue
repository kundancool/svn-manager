<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";

// Resizable two-pane split. Horizontal on wide layouts, stacks on narrow
// screens (drag disabled there). Left pane width persists per storageKey.

const props = defineProps<{ storageKey: string; initial?: number; min?: number; max?: number }>();

function load(): number {
  try {
    const v = Number(localStorage.getItem(`svnm-split-${props.storageKey}`));
    if (v > 0) return v;
  } catch { /* storage unavailable */ }
  return props.initial ?? 340;
}

const width = ref(load());
const dragging = ref(false);
let startX = 0;
let startW = 0;

function onDown(e: PointerEvent) {
  dragging.value = true;
  startX = e.clientX;
  startW = width.value;
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp, { once: true });
}

function onMove(e: PointerEvent) {
  const next = startW + (e.clientX - startX);
  width.value = Math.min(props.max ?? 640, Math.max(props.min ?? 240, next));
}

function onUp() {
  dragging.value = false;
  window.removeEventListener("pointermove", onMove);
  try {
    localStorage.setItem(`svnm-split-${props.storageKey}`, String(width.value));
  } catch { /* storage unavailable */ }
}

onBeforeUnmount(() => window.removeEventListener("pointermove", onMove));
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col md:flex-row" :class="dragging ? 'select-none' : ''">
    <section
      class="flex min-h-0 shrink-0 basis-2/5 flex-col overflow-hidden md:basis-auto"
      :style="{ width: undefined }"
      :class="'md:!basis-auto'"
      :data-w="width"
    >
      <div class="flex h-full min-h-0 flex-col md:hidden"><slot name="left" /></div>
      <div class="hidden h-full min-h-0 flex-col md:flex" :style="{ width: `${width}px` }">
        <slot name="left" />
      </div>
    </section>
    <div
      class="hidden w-[5px] shrink-0 cursor-col-resize border-l border-edge transition-colors hover:bg-mod/30 md:block"
      :class="dragging ? 'bg-mod/30' : ''"
      role="separator"
      aria-orientation="vertical"
      @pointerdown="onDown"
    ></div>
    <div class="min-h-0 min-w-0 flex-1 border-t border-edge md:border-t-0"><slot name="right" /></div>
  </div>
</template>
