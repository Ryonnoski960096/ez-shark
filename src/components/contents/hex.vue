<template>
  <div
    @input.prevent
    @beforeinput.prevent
    @paste.prevent
    contenteditable="true"
    v-bind="containerProps"
    class="h-500px w"
  >
    <!-- {{ list }} -->
    <div v-bind="wrapperProps">
      <Button type="primary" @click="copyContent(hexString)">复制Hex</Button>
      <div
        class="hex-cell h-35px"
        v-for="{ index, data } in list"
        :key="index + data.character_view"
      >
        <div class="address">
          {{ toHex8(data.offset_address) }}
        </div>
        <div class="hex-container">
          <span
            class="hex-byte"
            v-for="(bytes, i) in data.hex"
            :key="bytes + data.offset_address + i"
            >{{ bytes.toString(16) }}</span
          >
        </div>
        <div class="char-view">{{ data.character_view }}</div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { HexBody } from "@/stores/traffic";
import { toHex8 } from "@/utils/format";
import { copyContent } from "@/utils/tools";
import { useVirtualList } from "@vueuse/core";
import { Button } from "ant-design-vue";
import { computed, ref, watch } from "vue";

const { hexBody } = defineProps<{
  hexBody: HexBody[];
}>();
const hbList = ref(hexBody);
const { list, containerProps, wrapperProps } = useVirtualList(hbList, {
  itemHeight: 35,
  overscan: 4
});
watch(
  () => hexBody,
  () => {
    hbList.value = hexBody;
  }
);

const hexString = computed(() => {
  return hexBody
    .map((info) => {
      return info.hex
        .map((byte) => byte.toString(16).padStart(2, "0"))
        .join(" ");
    })
    .join("");
});
</script>
<style scoped>
.hex-cell {
  display: grid;
  grid-template-columns: 80px 1fr 150px;
  align-items: center;
  gap: 10px;
  padding: 8px;
  font-family: monospace;
}

.hex-container {
  display: grid;
  grid-template-columns: repeat(16, minmax(24px, auto));
  gap: 4px;
}

.hex-byte:hover,
.address:hover,
.char-view:hover {
  background-color: #dfdfdf;
}

.hex-byte {
  /* 减小内边距 */
  padding: 2px 2px;
  cursor: pointer;
  text-align: center;
  background-color: #fcfcfc;
  border-radius: 2px;
  max-width: 24px;
  overflow: hidden;
}

.char-view {
  color: #ac602d;
  min-width: 120px;
}
</style>
