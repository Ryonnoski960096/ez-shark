# UnoCSS Flex 布局规则使用指南

## 安装与配置

在 `uno.config.ts` 中引入规则：

```typescript
import { defineConfig } from "unocss";
import flexRules from "./flexRules";

export default defineConfig({
  rules: [...flexRules],
});
```

## 布局规则详解

### 1. 基础居中布局 `f-x`

| 类名  | 主轴对齐 | 交叉轴对齐 | 说明     |
| ----- | -------- | ---------- | -------- |
| `f-c` | 居中     | 居中       | 完全居中 |
| `f-s` | 起始位置 | 起始位置   | 起始对齐 |
| `f-b` | 两端对齐 | 居中       | 两端分散 |
| `f-r` | 结束位置 | 居中       | 尾部对齐 |
| `f-l` | 起始位置 | 居中       | 头部对齐 |
| `f-t` | 居中     | 顶部       | 垂直顶部 |
| `f-e` | 居中     | 底部       | 垂直底部 |

#### 示例

```html
<!-- 完全居中 -->
<div class="f-c">居中内容</div>

<!-- 两端对齐 -->
<div class="f-b">
  <div>左侧</div>
  <div>右侧</div>
</div>
```

### 2. 方向控制 `f-dir-x`

| 类名      | 说明           |
| --------- | -------------- |
| `f-dir-r` | 水平方向（行） |
| `f-dir-c` | 垂直方向（列） |

#### 示例

```html
<!-- 垂直居中，垂直方向 -->
<div class="f-c f-dir-c">
  <div>项目1</div>
  <div>项目2</div>
</div>
```

### 3. 复杂布局 `f-{方向}-{主轴对齐}-{交叉轴对齐}`

格式：`f-{row|col}-{对齐方式}-{可选交叉轴对齐}`

对齐方式：

- `start`：起始位置
- `center`：居中
- `end`：结束位置
- `between`：两端分散
- `around`：间隔分布

#### 示例

```html
<!-- 水平两端对齐，垂直居中 -->
<div class="f-row-between-center">
  <div>左侧</div>
  <div>右侧</div>
</div>

<!-- 垂直居中，底部对齐 -->
<div class="f-col-center-end">
  <div>顶部</div>
  <div>底部</div>
</div>
```

### 4. 换行控制 `f-wrap-x`

| 类名       | 说明   |
| ---------- | ------ |
| `f-wrap-n` | 不换行 |
| `f-wrap-w` | 换行   |

#### 示例

```html
<!-- 不换行 -->
<div class="f-wrap-n">
  <div>项目1</div>
  <div>项目2</div>
  <!-- 超出部分不会换行 -->
</div>

<!-- 换行 -->
<div class="f-wrap-w">
  <div>项目1</div>
  <div>项目2</div>
  <!-- 超出部分会换行 -->
</div>
```

### 5. 间距控制 `f-g-x`

| 类名     | 说明           |
| -------- | -------------- |
| `f-g-10` | 设置 10px 间距 |

#### 示例

```html
<!-- 设置 10px 间距 -->
<div class="f-c f-g-10">
  <div>项目1</div>
  <div>项目2</div>
  <div>项目3</div>
</div>
```

## 组合使用

```html
<!-- 复杂布局示例 -->
<div class="f-row-between-center f-g-20">
  <div>左侧</div>
  <div>中间</div>
  <div>右侧</div>
</div>
```
