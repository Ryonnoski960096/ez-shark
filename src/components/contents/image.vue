<template>
  <div class="image">
    <img :src="base64" alt="Image Preview" />
  </div>
</template>

<script setup lang="ts">
const { base64 } = defineProps<{
  base64: string;
}>();
</script>

<style scoped>
.image {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: auto;
}
</style>
