<script setup lang="ts">
import { ref } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import UiModal from "../ui/UiModal.vue";

const emit = defineEmits<{ close: [] }>();
const app = useAppStore();

const url = ref("");
const parent = ref("");
const name = ref("");
const busy = ref(false);

function suggestName(u: string): string {
  const parts = u.replace(/\/+$/, "").split("/");
  const last = parts[parts.length - 1] ?? "";
  return ["trunk", "branches", "tags"].includes(last) ? parts[parts.length - 2] ?? last : last;
}

async function pickParent() {
  const picked = await openDialog({ directory: true, title: "Choose where to check out" });
  if (typeof picked === "string") parent.value = picked;
}

async function checkout() {
  busy.value = true;
  try {
    const dest = `${parent.value.replace(/\/+$/, "")}/${name.value.trim()}`;
    const project = await api.checkoutProject(url.value.trim(), dest);
    app.project = project;
    await app.openProject(project.entry.local_path);
    app.toast("ok", "Checkout complete.");
    emit("close");
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <UiModal title="Check out a repository" width="min(520px, 94vw)" @close="emit('close')">
    <div class="flex flex-col gap-3">
      <label class="flex flex-col gap-1">
        <span class="microlabel">Repository URL</span>
        <input
          class="field font-mono"
          v-model="url"
          placeholder="https://svn.example.com/repo/trunk"
          @change="if (!name) name = suggestName(url);"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="microlabel">Check out into</span>
        <div class="flex gap-2">
          <input class="field flex-1 font-mono" v-model="parent" placeholder="Parent folder" />
          <button class="btn" @click="pickParent">Browse…</button>
        </div>
      </label>
      <label class="flex flex-col gap-1">
        <span class="microlabel">Folder name</span>
        <input class="field font-mono" v-model="name" placeholder="my-project" />
      </label>
      <p v-if="busy" class="text-[12.5px] text-faint">Checking out… this can take a while for large repositories.</p>
    </div>
    <template #footer>
      <button class="btn btn-ghost" :disabled="busy" @click="emit('close')">Cancel</button>
      <button class="btn btn-primary" :disabled="!url.trim() || !parent.trim() || !name.trim() || busy" @click="checkout">
        {{ busy ? "Checking out…" : "Check out" }}
      </button>
    </template>
  </UiModal>
</template>
