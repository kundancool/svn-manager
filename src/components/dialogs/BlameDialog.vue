<script setup lang="ts">
import { ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import { usePrefsStore } from "@/stores/prefs";
import type { BlameLine } from "@/lib/types";
import UiModal from "../ui/UiModal.vue";

const props = defineProps<{ path: string }>();
const emit = defineEmits<{ close: [] }>();

const app = useAppStore();
const prefs = usePrefsStore();
const lines = ref<BlameLine[]>([]);
const loading = ref(true);
const error = ref("");

// distinct-but-stable tint per revision so blocks are scannable
function revHue(rev: number | null): string {
  if (rev === null) return "transparent";
  const hue = (rev * 47) % 360;
  return `hsla(${hue}, 55%, 50%, 0.09)`;
}

api
  .wcBlame(app.localPath, props.path)
  .then((l) => (lines.value = l))
  .catch((e) => (error.value = errorMessage(e)))
  .finally(() => (loading.value = false));
</script>

<template>
  <UiModal :title="`Blame — ${path}`" width="min(900px, 96vw)" @close="emit('close')">
    <p v-if="loading" class="text-[12.5px] text-faint">Annotating…</p>
    <p v-else-if="error" class="text-[12.5px] text-del">{{ error }}</p>
    <div v-else class="max-h-[64vh] overflow-auto rounded-md border border-edge bg-bg">
      <table class="w-full border-collapse font-mono" :style="{ fontSize: `${prefs.diffFontPx}px` }">
        <tbody>
          <tr v-for="line in lines" :key="line.line_number" :style="{ background: revHue(line.revision) }">
            <td class="w-14 select-none border-r border-edge px-2 py-0 text-right text-accent">
              {{ line.revision ? `r${line.revision}` : "·" }}
            </td>
            <td class="w-24 select-none truncate border-r border-edge px-2 py-0 text-faint">
              {{ line.author ?? "" }}
            </td>
            <td class="w-10 select-none border-r border-edge px-2 py-0 text-right text-faint">
              {{ line.line_number }}
            </td>
            <td class="whitespace-pre px-2 py-0 text-ink">{{ line.text }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </UiModal>
</template>
