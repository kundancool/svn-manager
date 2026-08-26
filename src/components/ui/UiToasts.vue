<script setup lang="ts">
import { useAppStore } from "@/stores/app";

const app = useAppStore();
const tones = {
  ok: "border-ok/40 text-ok",
  error: "border-del/40 text-del",
  info: "border-edge text-ink",
};
</script>

<template>
  <div class="pointer-events-none fixed bottom-8 left-1/2 z-[60] flex -translate-x-1/2 flex-col items-center gap-2">
    <TransitionGroup name="toast">
      <div
        v-for="t in app.toasts"
        :key="t.id"
        class="vibrancy pointer-events-auto max-w-[70vw] truncate rounded-lg border bg-surface px-4 py-2 text-[12.5px] shadow-lg"
        :class="tones[t.tone]"
      >
        {{ t.text }}
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active, .toast-leave-active { transition: all 180ms ease; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateY(6px); }
</style>
