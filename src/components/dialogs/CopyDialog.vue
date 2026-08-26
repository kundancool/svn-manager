<script setup lang="ts">
import { computed, ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import UiModal from "../ui/UiModal.vue";

// Branches and tags are server-side copies of the WC's current URL.

const props = defineProps<{ mode: "branch" | "tag" }>();
const emit = defineEmits<{ close: []; created: [] }>();
const app = useAppStore();

const name = ref("");
const message = ref("");
const busy = ref(false);

const destination = computed(() =>
  `${props.mode === "branch" ? "branches" : "tags"}/${name.value.trim()}`
);

async function create() {
  busy.value = true;
  try {
    const msg = message.value.trim() || `Create ${props.mode} ${name.value.trim()}`;
    const rev = await api.createCopy(app.localPath, destination.value, msg);
    app.toast("ok", `Created ${destination.value}${rev ? ` in r${rev}` : ""}.`);
    emit("created");
    emit("close");
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <UiModal :title="mode === 'branch' ? 'New branch' : 'New tag'" @close="emit('close')">
    <div class="flex flex-col gap-3">
      <p class="text-[12.5px] leading-relaxed text-muted">
        Copies <span class="font-mono text-ink">{{ app.currentLocation }}</span> to
        <span class="font-mono text-ink">{{ destination }}</span> on the server.
      </p>
      <label class="flex flex-col gap-1">
        <span class="microlabel">Name</span>
        <input class="field font-mono" v-model="name" :placeholder="mode === 'branch' ? 'feature-x' : '1.2.0'" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="microlabel">Commit message</span>
        <input class="field" v-model="message" :placeholder="`Create ${mode} …`" />
      </label>
    </div>
    <template #footer>
      <button class="btn btn-ghost" :disabled="busy" @click="emit('close')">Cancel</button>
      <button class="btn btn-primary" :disabled="!name.trim() || busy" @click="create">
        {{ busy ? "Creating…" : mode === "branch" ? "Create branch" : "Create tag" }}
      </button>
    </template>
  </UiModal>
</template>
