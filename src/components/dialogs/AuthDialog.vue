<script setup lang="ts">
import { ref } from "vue";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import UiModal from "../ui/UiModal.vue";

// Prompted when svn rejects/misses auth. Optionally persists the login to
// the OS keychain so future operations authenticate silently.

const props = defineProps<{
  host: string;
  initialUsername?: string;
  busy?: boolean;
}>();
const emit = defineEmits<{ cancel: []; submit: [username: string, password: string] }>();

const app = useAppStore();
const username = ref(props.initialUsername ?? "");
const password = ref("");
const remember = ref(true);

async function submit() {
  if (!username.value || !password.value) return;
  if (remember.value) {
    try {
      app.config = await api.saveCredential(props.host, username.value, password.value);
    } catch (e) {
      app.toast("error", `Could not save to keychain: ${errorMessage(e)}`);
    }
  }
  emit("submit", username.value, password.value);
}
</script>

<template>
  <UiModal title="Sign in" @close="emit('cancel')">
    <div class="flex flex-col gap-3">
      <p class="text-[12.5px] leading-relaxed text-muted">
        <span class="font-mono text-ink">{{ host }}</span> needs your credentials.
      </p>
      <label class="flex flex-col gap-1">
        <span class="microlabel">Username</span>
        <input class="field" v-model="username" :disabled="busy" />
      </label>
      <label class="flex flex-col gap-1">
        <span class="microlabel">Password</span>
        <input class="field" type="password" v-model="password" :disabled="busy" @keydown.enter="submit" />
      </label>
      <label class="flex items-center gap-2 text-[12.5px] text-muted">
        <input type="checkbox" v-model="remember" />
        Save in the system keychain
      </label>
    </div>
    <template #footer>
      <button class="btn btn-ghost" :disabled="busy" @click="emit('cancel')">Cancel</button>
      <button class="btn btn-primary" :disabled="!username || !password || busy" @click="submit">
        {{ busy ? "Signing in…" : "Sign in and continue" }}
      </button>
    </template>
  </UiModal>
</template>
