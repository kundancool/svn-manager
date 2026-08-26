<script setup lang="ts">
defineProps<{ text: string }>();
</script>

<template>
  <div class="flex h-full items-center justify-center p-6">
    <p class="max-w-[340px] text-center text-[12.5px] leading-relaxed text-faint">{{ text }}</p>
  </div>
</template>
