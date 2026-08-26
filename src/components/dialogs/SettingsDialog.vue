<script setup lang="ts">
import { ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api, errorMessage } from "@/lib/api";
import { useAppStore } from "@/stores/app";
import { usePrefsStore } from "@/stores/prefs";
import { setTheme, themeMode, type ThemeMode } from "@/lib/theme";
import UiModal from "../ui/UiModal.vue";
import UiIcon from "../ui/UiIcon.vue";
import appIcon from "@/assets/icon.png";

const emit = defineEmits<{ close: [] }>();
const app = useAppStore();
const prefs = usePrefsStore();

type Tab = "general" | "preferences" | "credentials" | "about";
const tab = ref<Tab>("general");
const tabs: { id: Tab; label: string; icon: string }[] = [
  { id: "general", label: "General", icon: "gear" },
  { id: "preferences", label: "Preferences", icon: "sliders" },
  { id: "credentials", label: "Credentials", icon: "key" },
  { id: "about", label: "About", icon: "info" },
];

// general
const svnPath = ref(app.config?.svn_path ?? "");
const saving = ref(false);

async function saveSvnPath() {
  saving.value = true;
  try {
    app.svn = await api.setSvnPath(svnPath.value.trim() || null);
    await app.refreshConfig();
    app.toast(app.svn ? "ok" : "error", app.svn ? `Using svn ${app.svn.version}` : "svn not found with this setting");
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    saving.value = false;
  }
}

const themes: { value: ThemeMode; label: string; icon: string }[] = [
  { value: "light", label: "Light", icon: "sun" },
  { value: "system", label: "System", icon: "monitor" },
  { value: "dark", label: "Dark", icon: "moon" },
];

// credentials
const host = ref("");
const username = ref("");
const password = ref("");
const credBusy = ref(false);

async function addCredential() {
  credBusy.value = true;
  try {
    app.config = await api.saveCredential(host.value.trim(), username.value.trim(), password.value);
    host.value = username.value = password.value = "";
    app.toast("ok", "Credential saved to the system keychain.");
  } catch (e) {
    app.toast("error", errorMessage(e));
  } finally {
    credBusy.value = false;
  }
}

async function removeCredential(h: string) {
  try {
    app.config = await api.deleteCredential(h);
  } catch (e) {
    app.toast("error", errorMessage(e));
  }
}

// about
const version = ref("…");
getVersion().then((v) => (version.value = v)).catch(() => (version.value = "dev"));

const REPO_URL = "https://github.com/kundancool/svn-manager";

function link(url: string) {
  openUrl(url).catch((e) => app.toast("error", errorMessage(e)));
}
</script>

<template>
  <UiModal title="Settings" width="min(620px, 94vw)" @close="emit('close')">
    <div class="flex gap-4">
      <nav class="flex w-36 shrink-0 flex-col gap-0.5">
        <button
          v-for="t in tabs"
          :key="t.id"
          class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-[12.5px]"
          :class="tab === t.id ? 'bg-surface-2 font-semibold' : 'text-muted hover:bg-surface-2/60'"
          @click="tab = t.id"
        >
          <UiIcon :name="t.icon" :size="13" />{{ t.label }}
        </button>
      </nav>

      <div class="min-w-0 flex-1">
        <!-- General -->
        <div v-if="tab === 'general'" class="flex flex-col gap-5">
          <div class="flex flex-col gap-2">
            <span class="microlabel">Theme</span>
            <div class="flex gap-1.5">
              <button
                v-for="t in themes"
                :key="t.value"
                class="btn flex-1"
                :class="themeMode === t.value ? 'border-accent text-accent' : ''"
                @click="setTheme(t.value)"
              >
                <UiIcon :name="t.icon" />{{ t.label }}
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <span class="microlabel">Subversion binary</span>
            <p class="text-[12px] text-faint">
              {{ app.svn ? `Detected: ${app.svn.path} (v${app.svn.version})` : "svn was not detected." }}
              Leave empty to auto-detect.
            </p>
            <div class="flex gap-2">
              <input class="field flex-1 font-mono" v-model="svnPath" placeholder="/opt/homebrew/bin/svn" />
              <button class="btn" :disabled="saving" @click="saveSvnPath">Apply</button>
            </div>
          </div>
        </div>

        <!-- Preferences -->
        <div v-else-if="tab === 'preferences'" class="flex flex-col gap-4">
          <label class="flex items-start gap-2.5">
            <input
              type="checkbox"
              class="mt-0.5"
              :checked="prefs.showUnversioned"
              @change="prefs.set('showUnversioned', !prefs.showUnversioned)"
            />
            <span>
              <span class="block text-[13px]">Show unversioned files in Changes</span>
              <span class="block text-[11.5px] text-faint">Files svn doesn't track yet (marked ?)</span>
            </span>
          </label>

          <label class="flex items-start gap-2.5">
            <input
              type="checkbox"
              class="mt-0.5"
              :checked="prefs.confirmDestructive"
              @change="prefs.set('confirmDestructive', !prefs.confirmDestructive)"
            />
            <span>
              <span class="block text-[13px]">Confirm destructive actions</span>
              <span class="block text-[11.5px] text-faint">Ask before revert and rollback</span>
            </span>
          </label>

          <div class="flex flex-col gap-1.5">
            <span class="microlabel">Diff theme</span>
            <div class="grid grid-cols-5 gap-1.5">
              <button
                v-for="t in (['monokai', 'app', 'github', 'dracula', 'solarized'] as const)"
                :key="t"
                class="btn capitalize"
                :class="prefs.diffTheme === t ? 'border-accent text-accent' : ''"
                @click="prefs.set('diffTheme', t)"
              >{{ t }}</button>
            </div>
          </div>

          <div class="flex flex-col gap-1.5">
            <span class="microlabel">Diff text size</span>
            <div class="flex gap-1.5">
              <button
                v-for="s in (['small', 'medium', 'large'] as const)"
                :key="s"
                class="btn flex-1 capitalize"
                :class="prefs.diffFontSize === s ? 'border-accent text-accent' : ''"
                @click="prefs.set('diffFontSize', s)"
              >{{ s }}</button>
            </div>
          </div>

          <div class="flex flex-col gap-1.5">
            <span class="microlabel">History page size</span>
            <div class="flex gap-1.5">
              <button
                v-for="n in ([25, 50, 100] as const)"
                :key="n"
                class="btn flex-1"
                :class="prefs.historyPageSize === n ? 'border-accent text-accent' : ''"
                @click="prefs.set('historyPageSize', n)"
              >{{ n }}</button>
            </div>
          </div>
        </div>

        <!-- Credentials -->
        <div v-else-if="tab === 'credentials'" class="flex flex-col gap-4">
          <p class="text-[12px] leading-relaxed text-faint">
            Logins for SVN servers. Passwords are stored in the system keychain, never in
            configuration files, and are used automatically when a repository asks for them.
          </p>

          <ul v-if="app.config && app.config.credentials.length > 0" class="flex flex-col gap-1">
            <li
              v-for="c in app.config.credentials"
              :key="c.host"
              class="flex items-center gap-2 rounded-md border border-edge bg-surface-2/50 px-3 py-2"
            >
              <UiIcon name="key" class="text-faint" />
              <div class="min-w-0 flex-1">
                <div class="truncate font-mono text-[12px]">{{ c.host }}</div>
                <div class="text-[11.5px] text-faint">{{ c.username }}</div>
              </div>
              <button class="btn btn-ghost px-2 text-faint hover:text-del" :title="`Delete login for ${c.host}`" @click="removeCredential(c.host)">
                <UiIcon name="trash" />
              </button>
            </li>
          </ul>
          <p v-else class="text-[12.5px] text-faint">No saved logins yet.</p>

          <div class="flex flex-col gap-2 rounded-lg border border-edge p-3">
            <span class="microlabel">Add login</span>
            <input class="field font-mono" v-model="host" placeholder="Host — e.g. plugins.svn.wordpress.org" />
            <div class="flex gap-2">
              <input class="field" v-model="username" placeholder="Username" />
              <input class="field" type="password" v-model="password" placeholder="Password" @keydown.enter="addCredential" />
            </div>
            <button class="btn btn-primary self-start" :disabled="!host.trim() || !username.trim() || !password || credBusy" @click="addCredential">
              Save to keychain
            </button>
          </div>
        </div>

        <!-- About -->
        <div v-else class="flex flex-col items-center gap-4 py-4 text-center">
          <img :src="appIcon" alt="" class="h-24 w-24 drop-shadow-lg" draggable="false" />
          <div>
            <h3 class="text-[17px] font-bold">SVN Manager</h3>
            <p class="mt-0.5 font-mono text-[12px] text-muted">v{{ version }}</p>
          </div>
          <p class="max-w-[340px] text-[12.5px] leading-relaxed text-muted">
            A fast, open source, cross-platform Subversion client.
            Free software under the MIT license.
          </p>
          <div class="flex gap-2">
            <button class="btn" @click="link(REPO_URL)"><UiIcon name="globe" />GitHub</button>
            <button class="btn" @click="link(`${REPO_URL}/issues/new/choose`)">Report an issue</button>
            <button class="btn" @click="link(`${REPO_URL}/blob/main/LICENSE`)">License</button>
          </div>
          <div class="flex flex-col gap-0.5 text-[11.5px] text-faint">
            <span v-if="app.svn">Subversion {{ app.svn.version }} · {{ app.svn.path }}</span>
            <span>Built with Tauri, Vue, Tailwind CSS, and Lucide icons</span>
          </div>
        </div>
      </div>
    </div>
  </UiModal>
</template>
