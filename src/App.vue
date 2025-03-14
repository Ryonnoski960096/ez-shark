<template>
  <Suspense>
    <div class="h-100%">
      <TitleBar />
      <router-view />
    </div>
  </Suspense>
</template>
<script setup lang="ts">
import TitleBar from "@/components/layout/titleBar/index.vue";
</script>
<style>
.app {
  background-color: #f2f2f2;
  padding: 5px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  height: calc(100vh);
}
</style>
