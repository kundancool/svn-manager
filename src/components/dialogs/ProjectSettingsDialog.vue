<script setup lang="ts">
import { ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import type { RepoKind } from "@/lib/types";
import UiModal from "../ui/UiModal.vue";

const emit = defineEmits<{ close: [] }>();
const app = useAppStore();

const project = app.project!;
const publishEnabled = ref(project.entry.publish !== null || project.wporg !== null);
const slug = ref(project.entry.publish?.slug ?? project.wporg?.slug ?? project.entry.name);
const kind = ref<RepoKind>(project.entry.publish?.kind ?? project.wporg?.kind ?? "plugin");
const username = ref(project.entry.publish?.username ?? "");
const excludesText = ref((project.entry.publish?.excludes ?? []).join(", "));
const busy = ref(false);

async function save() {
  busy.value = true;
  try {
    const publish = publishEnabled.value
      ? {
          slug: slug.value.trim(),
          kind: kind.value,
          username: username.value.trim(),
          excludes: excludesText.value.split(",").map((s) => s.trim()).filter(Boolean),
        }
      : null;
    if (publish && !publish.slug) {
      app.toast("error", "Slug is required to enable publishing.");
      return;
    }
    app.config = await api.savePublish(project.entry.local_path, publish);
    await app.reloadProject();
    app.toast("ok", "Project settings saved.");
    emit("close");
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    busy.value = false;
  }
}

async function forget() {
  try {
    app.config = await api.forgetProject(project.entry.local_path);
    app.closeProject();
    emit("close");
  } catch (e) {
    app.toast("error", errorMessage(e));
  }
}
</script>

<template>
  <UiModal title="Project settings" width="min(520px, 94vw)" @close="emit('close')">
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="microlabel">Local path</span>
        <p class="truncate font-mono text-[12px] text-muted">{{ project.entry.local_path }}</p>
      </div>
      <div v-if="project.wc" class="flex flex-col gap-1">
        <span class="microlabel">Repository URL</span>
        <p class="truncate font-mono text-[12px] text-muted">{{ project.wc.url }}</p>
      </div>

      <div class="flex flex-col gap-3 rounded-lg border border-edge p-3">
        <label class="flex items-center gap-2 text-[13px] font-semibold">
          <input type="checkbox" v-model="publishEnabled" :disabled="project.wporg !== null" />
          WordPress.org publishing
        </label>
        <template v-if="publishEnabled">
          <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
            <label class="flex flex-col gap-1">
              <span class="microlabel">wp.org slug</span>
              <input class="field" v-model="slug" placeholder="my-plugin" />
            </label>
            <label class="flex flex-col gap-1">
              <span class="microlabel">Type</span>
              <select class="field" v-model="kind">
                <option value="plugin">Plugin</option>
                <option value="theme">Theme</option>
              </select>
            </label>
          </div>
          <label class="flex flex-col gap-1">
            <span class="microlabel">wp.org username</span>
            <input class="field" v-model="username" placeholder="optional — asked on publish" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="microlabel">Exclude patterns (.git, node_modules always excluded)</span>
            <input class="field" v-model="excludesText" placeholder="*.zip, src, tests" />
          </label>
          <p class="font-mono text-[11px] text-faint">
            https://{{ kind === "plugin" ? "plugins" : "themes" }}.svn.wordpress.org/{{ slug || "…" }}
          </p>
        </template>
      </div>

      <button class="btn btn-danger self-start" @click="forget">Remove from recent projects</button>
    </div>
    <template #footer>
      <button class="btn btn-ghost" @click="emit('close')">Cancel</button>
      <button class="btn btn-primary" :disabled="busy" @click="save">Save</button>
    </template>
  </UiModal>
</template>
