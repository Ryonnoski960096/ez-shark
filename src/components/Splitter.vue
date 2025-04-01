<template>
  <div
    ref="splitterRef"
    class="splitter"
    :class="[
      direction === 'vertical' ? 'splitter--vertical' : 'splitter--horizontal',
      { 'splitter--resizing': isResizing }
    ]"
    @mousedown="startResize"
  >
    <div class="splitter__line">
      <div class="dots" :class="{ 'dots--vertical': direction === 'vertical' }">
        <i class="point" />
        <i class="point" />
        <i class="point" />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, onUnmounted } from "vue";

interface Props {
  direction?: "vertical" | "horizontal";
  minSize?: number;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  direction: "vertical",
  minSize: 100,
  disabled: false
});
interface ResizeEvents {
  resizeStart: [];
  resizing: [sizes: { first: number; second: number }];
  resizeEnd: [sizes: { first: number; second: number }];
}

const emit = defineEmits<ResizeEvents>();

const splitterRef = ref<HTMLElement | null>(null);
const isResizing = ref(false);
const startPos = ref(0);
const firstElement = ref<HTMLElement | null>(null);
const secondElement = ref<HTMLElement | null>(null);
const initialSizes = ref({ first: 0, second: 0 });

const startResize = (e: MouseEvent) => {
  if (props.disabled) return;

  isResizing.value = true;
  startPos.value = props.direction === "vertical" ? e.clientX : e.clientY;

  if (firstElement.value && secondElement.value) {
    initialSizes.value = {
      first:
        props.direction === "vertical"
          ? firstElement.value.offsetWidth
          : firstElement.value.offsetHeight,
      second:
        props.direction === "vertical"
          ? secondElement.value.offsetWidth
          : secondElement.value.offsetHeight
    };
  }

  emit("resizeStart");
  document.addEventListener("mousemove", resize);
  document.addEventListener("mouseup", stopResize);

  // 添加禁止选择文本的类
  document.body.classList.add("no-select");
};

const resize = (e: MouseEvent) => {
  if (!isResizing.value) return;

  const currentPos = props.direction === "vertical" ? e.clientX : e.clientY;
  const delta = currentPos - startPos.value;

  if (firstElement.value && secondElement.value) {
    let newFirstSize = initialSizes.value.first + delta;
    let newSecondSize = initialSizes.value.second - delta;

    if (newFirstSize < props.minSize) {
      newFirstSize = props.minSize;
      newSecondSize =
        initialSizes.value.first + initialSizes.value.second - props.minSize;
    }

    if (newSecondSize < props.minSize) {
      newSecondSize = props.minSize;
      newFirstSize =
        initialSizes.value.first + initialSizes.value.second - props.minSize;
    }

    if (props.direction === "vertical") {
      firstElement.value.style.width = `${newFirstSize}px`;
      secondElement.value.style.width = `${newSecondSize}px`;
    } else {
      firstElement.value.style.height = `${newFirstSize}px`;
      secondElement.value.style.height = `${newSecondSize}px`;
    }

    emit("resizing", { first: newFirstSize, second: newSecondSize });
  }
};

const stopResize = () => {
  if (!isResizing.value) return;

  isResizing.value = false;
  document.removeEventListener("mousemove", resize);
  document.removeEventListener("mouseup", stopResize);

  // 移除禁止选择文本的类
  document.body.classList.remove("no-select");

  if (firstElement.value && secondElement.value) {
    const sizes = {
      first:
        props.direction === "vertical"
          ? firstElement.value.offsetWidth
          : firstElement.value.offsetHeight,
      second:
        props.direction === "vertical"
          ? secondElement.value.offsetWidth
          : secondElement.value.offsetHeight
    };
    emit("resizeEnd", sizes);
  }
};

const setElements = (first: HTMLElement, second: HTMLElement) => {
  firstElement.value = first;
  secondElement.value = second;
};

defineExpose({
  setElements
});

onUnmounted(() => {
  document.removeEventListener("mousemove", resize);
  document.removeEventListener("mouseup", stopResize);
  document.body.classList.remove("no-select");
});
</script>

<style>
/* 全局样式 */
.no-select {
  user-select: none !important;
}
</style>

<style scoped>
.splitter {
  position: relative;
  flex-shrink: 0;
  /* background: black; */
  z-index: 10;
}

/* 垂直分割线容器 */
.splitter--vertical {
  width: 8px;
  height: 100%; /* 垂直方向铺满 */
  cursor: col-resize;
  /* padding: 1px; */
  /* background-color: black; */
}

/* 水平分割线容器 */
.splitter--horizontal {
  width: 100%; /* 水平方向铺满 */
  height: 8px;
  cursor: row-resize;
}

.splitter__line {
  position: absolute;
  background: #e8e8e8;
  transition: background-color 0.2s;
  text-align: center;
}

/* 垂直分割线 */
.splitter--vertical .splitter__line {
  width: 100%; /* 填满容器宽度 */
  height: 100%;
  top: 0;
  left: 0;
}

/* 水平分割线 */
.splitter--horizontal .splitter__line {
  width: 100%;
  height: 100%; /* 填满容器高度 */
  left: 0;
  top: 0;
}

/* 鼠标悬停和拖动时的样式 */
.splitter:hover .splitter__line,
.splitter--resizing .splitter__line {
  background: #1890ff;
}

/* 拖动时的样式 */
.splitter--resizing {
  background-color: rgba(24, 144, 255, 0.1);
}

.point {
  width: 5px;
  height: 5px;
  background: #979797;
  border-radius: 555px;
}

.dots {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  font-size: 7px;
  line-height: 1;
}

/* 水平方向的点样式 */
.dots i {
  display: inline-block;
  margin: 0 1px;
}

/* 垂直方向的点样式 */
.dots--vertical {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
}

.dots--vertical i {
  display: block;
  line-height: 8px; /* 调整垂直间距 */
}

/* 鼠标悬停和拖动时的点颜色 */
.splitter:hover .dots,
.splitter--resizing .dots {
  color: #fff;
}
</style>
