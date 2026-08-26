<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import UiIcon from "./UiIcon.vue";

export interface MenuItem {
  id: string;
  label: string;
  icon?: string;
  danger?: boolean;
}

const props = defineProps<{ x: number; y: number; items: MenuItem[] }>();
const emit = defineEmits<{ pick: [id: string]; close: [] }>();

const el = ref<HTMLElement | null>(null);
const pos = ref({ left: props.x, top: props.y });

function onGlobal(e: MouseEvent) {
  if (el.value && !el.value.contains(e.target as Node)) emit("close");
}
function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => {
  // keep the menu inside the window
  requestAnimationFrame(() => {
    const r = el.value?.getBoundingClientRect();
    if (!r) return;
    pos.value = {
      left: Math.min(props.x, window.innerWidth - r.width - 8),
      top: Math.min(props.y, window.innerHeight - r.height - 8),
    };
  });
  window.addEventListener("mousedown", onGlobal, true);
  window.addEventListener("keydown", onKey);
});
onBeforeUnmount(() => {
  window.removeEventListener("mousedown", onGlobal, true);
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div
    ref="el"
    class="vibrancy fixed z-[70] min-w-44 rounded-lg border border-edge bg-surface py-1 shadow-2xl"
    :style="{ left: `${pos.left}px`, top: `${pos.top}px` }"
    role="menu"
  >
    <button
      v-for="item in items"
      :key="item.id"
      class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-[12.5px] hover:bg-surface-2"
      :class="item.danger ? 'text-del' : 'text-ink'"
      role="menuitem"
      @click="emit('pick', item.id)"
    >
      <UiIcon v-if="item.icon" :name="item.icon" :size="13" class="text-faint" />
      {{ item.label }}
    </button>
  </div>
</template>
