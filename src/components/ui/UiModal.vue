<script setup lang="ts">
import UiIcon from "./UiIcon.vue";

defineProps<{ title: string; width?: string }>();
const emit = defineEmits<{ close: [] }>();
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-[var(--scrim)]"
    role="dialog"
    aria-modal="true"
    :aria-label="title"
    @keydown.esc="emit('close')"
  >
    <div
      class="vibrancy flex max-h-[85vh] flex-col rounded-xl border border-edge bg-surface shadow-2xl"
      :style="{ width: width ?? 'min(440px, 92vw)' }"
    >
      <header class="flex items-center justify-between border-b border-edge px-4 py-3">
        <h2 class="text-[14px] font-semibold">{{ title }}</h2>
        <button class="btn btn-ghost -mr-1 px-1.5 py-1 text-faint" aria-label="Close" @click="emit('close')">
          <UiIcon name="x" />
        </button>
      </header>
      <div class="min-h-0 flex-1 overflow-y-auto p-4">
        <slot />
      </div>
      <footer v-if="$slots.footer" class="flex justify-end gap-2 border-t border-edge px-4 py-3">
        <slot name="footer" />
      </footer>
    </div>
  </div>
</template>
