<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";

// Vertical two-pane split (top/bottom) with a draggable divider; top height
// persists per storageKey.

const props = defineProps<{ storageKey: string; initial?: number; min?: number; max?: number }>();

function load(): number {
  try {
    const v = Number(localStorage.getItem(`svnm-vsplit-${props.storageKey}`));
    if (v > 0) return v;
  } catch { /* storage unavailable */ }
  return props.initial ?? 280;
}

const height = ref(load());
const dragging = ref(false);
let startY = 0;
let startH = 0;

function onDown(e: PointerEvent) {
  dragging.value = true;
  startY = e.clientY;
  startH = height.value;
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp, { once: true });
}

function onMove(e: PointerEvent) {
  const next = startH + (e.clientY - startY);
  height.value = Math.min(props.max ?? 600, Math.max(props.min ?? 140, next));
}

function onUp() {
  dragging.value = false;
  window.removeEventListener("pointermove", onMove);
  try {
    localStorage.setItem(`svnm-vsplit-${props.storageKey}`, String(height.value));
  } catch { /* storage unavailable */ }
}

onBeforeUnmount(() => window.removeEventListener("pointermove", onMove));
</script>

<template>
  <div class="flex h-full min-h-0 flex-col" :class="dragging ? 'select-none' : ''">
    <div class="flex min-h-0 shrink-0 flex-col overflow-hidden" :style="{ height: `${height}px` }">
      <slot name="top" />
    </div>
    <div
      class="h-[5px] shrink-0 cursor-row-resize border-t border-edge transition-colors hover:bg-mod/30"
      :class="dragging ? 'bg-mod/30' : ''"
      role="separator"
      aria-orientation="horizontal"
      @pointerdown="onDown"
    ></div>
    <div class="min-h-0 flex-1"><slot name="bottom" /></div>
  </div>
</template>
