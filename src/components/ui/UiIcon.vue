<script setup lang="ts">
import type { FunctionalComponent } from "vue";
import {
  ArrowLeft,
  Ban,
  Bug,
  FileClock,
  Lock,
  LockOpen,
  Search,
  Wrench,
  ChevronRight,
  Check,
  Clock,
  Download,
  File,
  FileArchive,
  FileAudio,
  FileCode,
  FileCog,
  FileImage,
  FileJson,
  FileText,
  FileVideo,
  Folder,
  WrapText,
  FolderOpen,
  GitBranch,
  GitMerge,
  Globe,
  Info,
  KeyRound,
  Minus,
  Monitor,
  Moon,
  Plus,
  RefreshCw,
  Settings,
  SlidersHorizontal,
  Sun,
  Tag,
  Trash2,
  Undo2,
  Upload,
  X,
} from "lucide-vue-next";

// Single icon gateway — every icon in the app goes through this name map so
// usage stays consistent and swapping the icon set is a one-file change.
const icons: Record<string, FunctionalComponent> = {
  back: ArrowLeft,
  ban: Ban,
  bug: Bug,
  "file-clock": FileClock,
  lock: Lock,
  "lock-open": LockOpen,
  search: Search,
  wrench: Wrench,
  branch: GitBranch,
  check: Check,
  "chevron-right": ChevronRight,
  clock: Clock,
  download: Download,
  file: File,
  "file-archive": FileArchive,
  "file-audio": FileAudio,
  "file-code": FileCode,
  "file-cog": FileCog,
  "file-image": FileImage,
  "file-json": FileJson,
  "file-text": FileText,
  "file-video": FileVideo,
  "wrap-text": WrapText,
  folder: Folder,
  "folder-open": FolderOpen,
  gear: Settings,
  globe: Globe,
  info: Info,
  key: KeyRound,
  merge: GitMerge,
  minus: Minus,
  monitor: Monitor,
  moon: Moon,
  plus: Plus,
  refresh: RefreshCw,
  sliders: SlidersHorizontal,
  sun: Sun,
  tag: Tag,
  trash: Trash2,
  undo: Undo2,
  upload: Upload,
  x: X,
};

defineProps<{ name: string; size?: number }>();
</script>

<template>
  <component
    :is="icons[name] ?? File"
    :size="size ?? 14"
    :stroke-width="1.8"
    aria-hidden="true"
    class="shrink-0"
  />
</template>
