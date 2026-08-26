<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import type { CommandLogEntry } from "@/lib/types";
import UiModal from "../ui/UiModal.vue";
import UiIcon from "../ui/UiIcon.vue";

// CLI-style console of every svn command the app ran, with output.
// Passwords never appear here — they are passed to svn over stdin.

const emit = defineEmits<{ close: [] }>();
const app = useAppStore();

const logs = ref<CommandLogEntry[]>([]);
const loading = ref(true);

async function refresh() {
  try {
    logs.value = await api.getDebugLogs();
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    loading.value = false;
  }
}

async function clear() {
  await api.clearDebugLogs();
  await refresh();
}

function fmtTime(ms: number): string {
  return new Date(ms).toLocaleTimeString(undefined, { hour12: false });
}

const timer = setInterval(refresh, 2000);
onBeforeUnmount(() => clearInterval(timer));
refresh();
</script>

<template>
  <UiModal title="Debug console" width="min(860px, 96vw)" @close="emit('close')">
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <span class="text-[12px] text-faint">
          Every svn command this session ({{ logs.length }}) — newest last. Passwords go over stdin and are never shown.
        </span>
        <div class="flex gap-1.5">
          <button class="btn btn-ghost px-2 text-muted" title="Refresh" @click="refresh"><UiIcon name="refresh" :size="13" /></button>
          <button class="btn btn-ghost px-2 text-muted hover:text-del" title="Clear log" @click="clear"><UiIcon name="trash" :size="13" /></button>
        </div>
      </div>

      <div class="max-h-[62vh] overflow-y-auto rounded-md border border-edge bg-[#14171d] p-3 font-mono text-[11.5px] leading-relaxed">
        <p v-if="loading" class="text-[#8b95a5]">Loading…</p>
        <p v-else-if="logs.length === 0" class="text-[#8b95a5]">No commands run yet.</p>
        <div v-for="(entry, i) in logs" :key="i" class="mb-3">
          <div class="flex items-baseline gap-2">
            <span class="shrink-0 text-[#5d6675]">{{ fmtTime(entry.at) }}</span>
            <span class="break-all text-[#d7dce4]">$ {{ entry.command }}<template v-if="entry.had_stdin"> <span class="text-[#e8a13c]">&lt;stdin&gt;</span></template></span>
          </div>
          <div v-if="entry.cwd" class="pl-[74px] text-[#5d6675]">cwd: {{ entry.cwd }}</div>
          <div class="pl-[74px]" :class="entry.ok ? 'text-[#67b26f]' : 'text-[#d96c6c]'">
            exit {{ entry.exit_code ?? "—" }} · {{ entry.duration_ms }}ms
          </div>
          <pre v-if="entry.stderr" class="mt-0.5 whitespace-pre-wrap break-all pl-[74px] text-[#d96c6c]">{{ entry.stderr }}</pre>
          <pre v-if="entry.stdout" class="mt-0.5 max-h-40 overflow-y-auto whitespace-pre-wrap break-all pl-[74px] text-[#8b95a5]">{{ entry.stdout }}</pre>
        </div>
      </div>
    </div>
  </UiModal>
</template>
