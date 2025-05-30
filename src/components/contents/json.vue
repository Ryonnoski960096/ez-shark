<template>
  <div
    @input.prevent
    @beforeinput.prevent
    @paste.prevent
    contenteditable="true"
    v-html="readOnlyValue"
    style="font-size: 12px"
    v-if="readOnly"
  />
  <JsonEditorVue
    v-else
    ref="jsonEditorVueRef"
    :readOnly="readOnly"
    v-model="value"
    style="font-size: 12px"
    v-bind="JsonEditorVueProps"
  />
</template>

<script lang="ts" setup>
import type { ActiveTraffic } from "@/utils/eventBus";
import { escapeHtml, generateMarkStyle } from "@/utils/format";
import { waitForCondition } from "@/utils/tools";
import JsonEditorVue from "json-editor-vue";
import { nextTick, ref, watch } from "vue";

const value = defineModel<string>("value", {
  required: true,
  default: ""
});

const { readOnly = false, highlightNodeId = "json-mask" } = defineProps<{
  readOnly?: boolean;
  highlightNodeId?: string;
}>();

enum Mode {
  text = "text",
  tree = "tree",
  table = "table"
}

const JsonEditorVueProps = {
  mode: Mode.text,
  mainMenuBar: false,
  statusBar: false,
  navigationBar: false,
  askToFormat: !readOnly,
  flattenColumns: true,
  style: {
    height: "100%"
  }
};

const jsonEditorVueRef = ref();

const readOnlyValue = ref<string>();
const formatting = ref<boolean>(false);
// 保存最原始的JSON字符串，用于重新格式化
const originalJsonText = ref<string>("");
// 记录是否正在高亮中
const isHighlighting = ref<boolean>(false);

/**
 * 格式化JSON字符串并更新视图
 * @param newValue - 要格式化的JSON字符串
 * @returns 格式化后的HTML字符串
 */
const formatValue = (newValue: string): string => {
  // console.log("formatValue", newValue);
  formatting.value = true;

  // 保存原始JSON文本以便后续使用
  if (!isHighlighting.value) {
    originalJsonText.value = newValue;
  }

  let formattedString: string = "";
  try {
    const parsedValue = JSON.parse(newValue);
    formattedString += `<span style="color: gray;">{</span><br/>&nbsp;&nbsp;`;

    const keys: string[] = Object.keys(parsedValue);
    keys.forEach((key: string, index: number) => {
      formattedString += `<span style="color: black;">"${key}"</span>`;
      formattedString += `<span style="color: gray;">:</span>`;

      const val = parsedValue[key];
      switch (typeof val) {
        case "string":
          // 对字符串处理已转义
          formattedString += `<span style="color: green;">"${
            val
              .replace(/\\/g, "\\\\") // 转义反斜杠
              .replace(/"/g, '\\\\"') // 处理双引号
            // 这里也可以处理其他需要转义的特殊字符
            // .replace(/\n/g, '\\n') // 举例处理换行符
          }"</span>`;
          break;
        case "number":
          formattedString += `<span style="color: #BB6A77;">${val}</span>`;
          break;
        case "boolean":
          formattedString += `<span style="color: blue;">${val}</span>`;
          break;
        case "object":
          if (val === null) {
            formattedString += `<span style="color: red;">null</span>`;
          } else if (Array.isArray(val)) {
            formattedString += `<span style="color: gray;">[</span>`;
            formattedString += `<br/>&nbsp;&nbsp;`;
            // 处理数组内容
            val.forEach((item: any, idx: number) => {
              if (idx > 0) {
                formattedString += `<span style="color: gray;">,</span><br/>&nbsp;&nbsp;`; // 添加逗号分隔
              }
              formattedString += formatValue(JSON.stringify(item)); // 递归格式化数组项
            });
            formattedString += `<br/>&nbsp;&nbsp;<span style="color: gray;">]</span>`;
          } else {
            formattedString += `<span style="color: gray;">{</span>`;
            formattedString += `<br/>&nbsp;&nbsp;`;
            formattedString += formatValue(JSON.stringify(val)); // 递归处理对象
            formattedString += `<br/>&nbsp;&nbsp;<span style="color: gray;">}</span>`;
          }
          break;
        default:
          formattedString += `<span style="color: red;">${String(val)}</span>`;
          break;
      }

      // 在不是最后一个键值对时添加逗号
      if (index < keys.length - 1) {
        formattedString += `<span style="color: gray;">,</span>`; // 添加逗号
      }

      formattedString += `<br/>&nbsp;&nbsp;`; // 添加行间距
    });

    formattedString += `<span style="color: gray;">}</span>`; // 关闭大括号
  } catch {
    formattedString += `<span style="color: red;">${newValue}</span>`; // 处理 JSON 解析错误
  }

  readOnlyValue.value = formattedString;
  formatting.value = false;

  return formattedString;
};

/**
 * 工具函数：仅格式化JSON，不更新状态
 * @param newValue - 要格式化的JSON字符串
 * @returns 格式化后的HTML字符串
 */
const formatValueOnly = (newValue: string): string => {
  let formattedString: string = "";
  try {
    const parsedValue = JSON.parse(newValue);
    formattedString += `<span style="color: gray;">{</span><br/>&nbsp;&nbsp;`;

    const keys: string[] = Object.keys(parsedValue);
    keys.forEach((key: string, index: number) => {
      formattedString += `<span style="color: black;">"${key}"</span>`;
      formattedString += `<span style="color: gray;">:</span>`;

      const val = parsedValue[key];
      switch (typeof val) {
        case "string":
          formattedString += `<span style="color: green;">"${val
            .replace(/\\/g, "\\\\")
            .replace(/"/g, '\\\\"')}"</span>`;
          break;
        case "number":
          formattedString += `<span style="color: #BB6A77;">${val}</span>`;
          break;
        case "boolean":
          formattedString += `<span style="color: blue;">${val}</span>`;
          break;
        case "object":
          if (val === null) {
            formattedString += `<span style="color: red;">null</span>`;
          } else if (Array.isArray(val)) {
            formattedString += `<span style="color: gray;">[</span>`;
            formattedString += `<br/>&nbsp;&nbsp;`;
            val.forEach((item: any, idx: number) => {
              if (idx > 0) {
                formattedString += `<span style="color: gray;">,</span><br/>&nbsp;&nbsp;`;
              }
              formattedString += formatValueOnly(JSON.stringify(item));
            });
            formattedString += `<br/>&nbsp;&nbsp;<span style="color: gray;">]</span>`;
          } else {
            formattedString += `<span style="color: gray;">{</span>`;
            formattedString += `<br/>&nbsp;&nbsp;`;
            formattedString += formatValueOnly(JSON.stringify(val));
            formattedString += `<br/>&nbsp;&nbsp;<span style="color: gray;">}</span>`;
          }
          break;
        default:
          console.error("default", val);
          formattedString += `<span style="color: red;">${String(val)}</span>`;
          break;
      }

      if (index < keys.length - 1) {
        formattedString += `<span style="color: gray;">,</span>`;
      }

      formattedString += `<br/>&nbsp;&nbsp;`;
    });

    formattedString += `<span style="color: gray;">}</span>`;
  } catch {
    formattedString += `<span style="color: red;">${newValue}</span>`;
  }

  return formattedString;
};

watch(value, (newValue: string) => {
  // 只有在非高亮过程中才进行普通格式化
  if (!isHighlighting.value) {
    formatValue(escapeHtml(newValue));
  }
});

/**
 * 在原始文本中查找指定位置的关键字
 * @param text - 要搜索的文本
 * @param keyword - 要查找的关键字
 * @param exactIndex - 精确的索引位置
 * @returns 如果在指定位置找到关键字则返回true，否则返回false
 */
const findExactKeywordAtPosition = (
  text: string,
  keyword: string,
  exactIndex: number
): boolean => {
  // 验证索引位置确实匹配关键字
  if (
    exactIndex >= 0 &&
    exactIndex + keyword.length <= text.length &&
    text.substring(exactIndex, exactIndex + keyword.length) === keyword
  ) {
    return true;
  }
  return false;
};

/**
 * 直接应用高亮到指定文本位置
 * @param originalText - 原始文本
 * @param keyword - 要高亮的关键字
 * @param exactIndex - 精确的索引位置
 * @returns 添加了高亮标记的文本
 * @throws 如果指定位置没有找到关键字，则抛出错误
 */
const directHighlightMethod = (
  originalText: string,
  keyword: string,
  exactIndex: number
): string => {
  // 验证索引位置
  if (!findExactKeywordAtPosition(originalText, keyword, exactIndex)) {
    throw new Error("Keyword not found at specified position");
  }

  // 直接分割并高亮
  const before: string = originalText.substring(0, exactIndex);
  const after: string = originalText.substring(exactIndex + keyword.length);

  return (
    before +
    `<mask id='${highlightNodeId}' style='${generateMarkStyle({})}'>${keyword}</mask>` +
    after
  );
};
/**
 * 高亮指定流量中的关键字
 * @param activeTraffic - 活动流量对象，包含要高亮的信息
 */
/**
 * 高亮指定流量中的关键字
 * @param activeTraffic - 活动流量对象，包含要高亮的信息
 */
/**
 * 高亮指定流量中的关键字
 * @param activeTraffic - 活动流量对象，包含要高亮的信息
 */
const highlight = async (activeTraffic: ActiveTraffic): Promise<void> => {
  if (!activeTraffic || activeTraffic.index === undefined) return;
  // 等待format完成
  await waitForCondition(() => !formatting.value);
  isHighlighting.value = true;

  try {
    // 清除之前的高亮
    const previousHighlight = document.getElementById(highlightNodeId);
    if (previousHighlight) {
      // 获取高亮元素的文本内容
      const highlightedText = previousHighlight.textContent || "";

      // 替换高亮元素为纯文本
      const textNode = document.createTextNode(highlightedText);
      previousHighlight.parentNode?.replaceChild(textNode, previousHighlight);

      // 合并相邻的文本节点
      if (textNode.parentNode) {
        textNode.parentNode.normalize();
      }
    }

    // 获取原始JSON文本 - 从value中获取，而不是从可能已经包含高亮标记的DOM中获取
    const jsonText: string = escapeHtml(value.value || "");

    // 首先验证指定位置是否确实有关键字
    const isValidHighlight: boolean = findExactKeywordAtPosition(
      jsonText,
      activeTraffic.keyword,
      activeTraffic.index
    );

    if (!isValidHighlight) {
      console.warn("Keyword not found at specified position:", activeTraffic);
      return; // 如果指定位置没有关键字，不进行高亮
    }

    // 检查是否是特殊字符
    const isSpecialChar = /[{}[\]",:0-9]|true|false|null/g.test(
      activeTraffic.keyword
    );
    // console.log("previousHighlight", previousHighlight);

    // 如果已经有高亮元素，我们需要使用不同的策略
    if (previousHighlight) {
      // 重新格式化原始JSON，然后尝试在格式化后的内容中查找并高亮
      const formattedHtml: string = formatValueOnly(jsonText);

      // 创建临时DOM元素
      const tempDiv: HTMLDivElement = document.createElement("div");
      tempDiv.innerHTML = formattedHtml;

      // 查找所有文本节点
      let textNodes: Text[] = [];
      const getTextNodes = (node: Node) => {
        if (node.nodeType === Node.TEXT_NODE) {
          textNodes.push(node as Text);
        } else {
          for (let i = 0; i < node.childNodes.length; i++) {
            getTextNodes(node.childNodes[i]);
          }
        }
      };

      getTextNodes(tempDiv);

      // 在原始JSON中查找关键字的所有出现位置
      const allOccurrences: number[] = [];
      let searchIndex = 0;
      while (true) {
        const foundIndex = jsonText.indexOf(activeTraffic.keyword, searchIndex);
        if (foundIndex === -1) break;
        allOccurrences.push(foundIndex);
        searchIndex = foundIndex + 1;
      }

      // 找到匹配当前索引的出现位置
      const targetOccurrenceIndex = allOccurrences.indexOf(activeTraffic.index);

      if (targetOccurrenceIndex !== -1) {
        // 在格式化后的文本中查找第n次出现的关键字
        let occurrenceCount = 0;
        let found = false;

        for (let textNode of textNodes) {
          const text = textNode.textContent || "";
          let nodeSearchIndex = 0;

          while (true) {
            const nodeFoundIndex = text.indexOf(
              activeTraffic.keyword,
              nodeSearchIndex
            );
            if (nodeFoundIndex === -1) break;

            if (occurrenceCount === targetOccurrenceIndex) {
              // 找到了目标出现位置
              const before = text.substring(0, nodeFoundIndex);
              const matched = text.substring(
                nodeFoundIndex,
                nodeFoundIndex + activeTraffic.keyword.length
              );
              const after = text.substring(
                nodeFoundIndex + activeTraffic.keyword.length
              );

              const parent = textNode.parentNode;
              if (parent) {
                const beforeNode = document.createTextNode(before);

                const highlightNode = document.createElement("mask");
                highlightNode.id = highlightNodeId;
                highlightNode.setAttribute("style", generateMarkStyle({}));
                highlightNode.textContent = matched;

                const afterNode = document.createTextNode(after);

                parent.insertBefore(beforeNode, textNode);
                parent.insertBefore(highlightNode, textNode);
                parent.insertBefore(afterNode, textNode);
                parent.removeChild(textNode);

                found = true;
                break;
              }
            }

            occurrenceCount++;
            nodeSearchIndex = nodeFoundIndex + 1;
          }

          if (found) break;
        }

        if (found) {
          readOnlyValue.value = tempDiv.innerHTML;
        } else {
          // 如果无法找到对应的出现位置，回退到完全重新格式化
          formatValue(jsonText);
          // 然后重新尝试高亮
          setTimeout(() => highlight(activeTraffic), 0);
          return;
        }
      } else {
        // 无法找到对应的出现位置，回退到完全重新格式化
        formatValue(jsonText);
        // 然后重新尝试高亮
        setTimeout(() => highlight(activeTraffic), 0);
        return;
      }
    } else if (isSpecialChar) {
      // 对于特殊字符，使用预格式化后直接插入高亮标记的方法
      try {
        // 先格式化原始JSON
        const formattedHtml: string = formatValueOnly(jsonText);

        // 创建临时DOM元素
        const tempDiv: HTMLDivElement = document.createElement("div");
        tempDiv.innerHTML = formattedHtml;

        // 查找纯文本内容
        let textNodes: Text[] = [];
        const getTextNodes = (node: Node) => {
          if (node.nodeType === Node.TEXT_NODE) {
            textNodes.push(node as Text);
          } else {
            for (let i = 0; i < node.childNodes.length; i++) {
              getTextNodes(node.childNodes[i]);
            }
          }
        };

        getTextNodes(tempDiv);

        // 尝试在文本节点中找到特殊字符
        let found = false;
        for (let textNode of textNodes) {
          const text = textNode.textContent || "";
          const index = text.indexOf(activeTraffic.keyword);

          if (index !== -1) {
            // 找到了特殊字符
            const before = text.substring(0, index);
            const matched = text.substring(
              index,
              index + activeTraffic.keyword.length
            );
            const after = text.substring(index + activeTraffic.keyword.length);

            const parent = textNode.parentNode;
            if (parent) {
              const beforeNode = document.createTextNode(before);

              const highlightNode = document.createElement("mask");
              highlightNode.id = highlightNodeId;
              highlightNode.setAttribute("style", generateMarkStyle({}));
              highlightNode.textContent = matched;

              const afterNode = document.createTextNode(after);

              parent.insertBefore(beforeNode, textNode);
              parent.insertBefore(highlightNode, textNode);
              parent.insertBefore(afterNode, textNode);
              parent.removeChild(textNode);

              found = true;
              break;
            }
          }
        }

        if (found) {
          readOnlyValue.value = tempDiv.innerHTML;
        } else {
          throw new Error("Special character not found in formatted HTML");
        }
      } catch (error) {
        console.warn("Special character highlight failed:", error);

        // 回退方法：使用预渲染的HTML字符串
        try {
          // 先格式化不包含高亮的JSON
          formatValue(jsonText);

          // 然后在已格式化的HTML中查找并替换特殊字符
          await nextTick();

          const container = document.querySelector('[contenteditable="true"]');
          if (container) {
            // 使用DOM API查找文本节点
            const walker = document.createTreeWalker(
              container,
              NodeFilter.SHOW_TEXT,
              null
            );

            let node;
            while ((node = walker.nextNode())) {
              const text = node.textContent || "";
              const index = text.indexOf(activeTraffic.keyword);

              if (index !== -1) {
                // 找到了特殊字符
                const range = document.createRange();
                range.setStart(node, index);
                range.setEnd(node, index + activeTraffic.keyword.length);

                // 创建高亮元素
                const highlightEl = document.createElement("mask");
                highlightEl.id = highlightNodeId;
                highlightEl.setAttribute("style", generateMarkStyle({}));
                highlightEl.textContent = activeTraffic.keyword;

                // 替换选中的文本
                range.deleteContents();
                range.insertNode(highlightEl);

                break;
              }
            }
          }
        } catch (finalError) {
          console.error(
            "All special character highlight methods failed:",
            finalError
          );
          // 最后的回退：至少显示格式化的JSON
          formatValue(jsonText);
        }
      }
    } else {
      // 非特殊字符，使用原有的高亮方法
      try {
        // 使用直接替换方法高亮
        const highlightedText: string = directHighlightMethod(
          jsonText,
          activeTraffic.keyword,
          activeTraffic.index
        );

        // 格式化高亮后的文本
        formatValue(highlightedText);
      } catch (error) {
        console.warn("Direct highlight method failed:", error);

        // 方法2：尝试在格式化后的JSON中定位并高亮
        try {
          // 格式化原始JSON
          const formattedHtml: string = formatValueOnly(jsonText);

          // 在原始文本和格式化文本之间建立字符映射并高亮
          const exactHighlightedHtml: string = exactPositionHighlight(
            formattedHtml,
            jsonText,
            activeTraffic.keyword,
            activeTraffic.index
          );

          readOnlyValue.value = exactHighlightedHtml;
        } catch (innerError) {
          console.warn("Exact position highlight failed:", innerError);

          // 最后的回退方法：使用分片法
          try {
            // 确保精确分片
            const exactStart: string = jsonText.substring(
              0,
              activeTraffic.index
            );
            const exactKeyword: string = jsonText.substring(
              activeTraffic.index,
              activeTraffic.index + activeTraffic.keyword.length
            );
            const exactEnd: string = jsonText.substring(
              activeTraffic.index + activeTraffic.keyword.length
            );

            // 验证分片是否准确
            if (exactKeyword !== activeTraffic.keyword) {
              throw new Error("Keyword verification failed in fallback method");
            }

            const highlightStr: string =
              escapeHtml(exactStart) +
              `<mask id='${highlightNodeId}' style='${generateMarkStyle({})}'>${escapeHtml(exactKeyword)}</mask>` +
              escapeHtml(exactEnd);

            formatValue(highlightStr);
          } catch (finalError) {
            console.error("All highlight methods failed:", finalError);
            // 如果所有方法都失败，至少保证正常显示JSON
            formatValue(jsonText);
          }
        }
      }
    }

    await nextTick();
    const maskElement: HTMLElement | null =
      document.getElementById(highlightNodeId);
    if (maskElement) {
      // 平滑滚动到高亮元素
      maskElement.scrollIntoView({
        behavior: "smooth",
        block: "center"
      });
    }
  } finally {
    // 确保不管发生什么，都重置高亮状态
    isHighlighting.value = false;
  }
};
/**
 * 根据精确的字符位置在格式化的HTML中高亮关键字
 * @param formattedHtml - 格式化后的HTML
 * @param originalText - 原始文本
 * @param keyword - 要高亮的关键字
 * @param exactIndex - 精确的索引位置
 * @returns 添加了高亮标记的HTML
 * @throws 如果无法在格式化HTML中找到对应位置，则抛出错误
 */
const exactPositionHighlight = (
  formattedHtml: string,
  originalText: string,
  keyword: string,
  exactIndex: number
): string => {
  // 先验证原始位置是否有关键字
  if (!findExactKeywordAtPosition(originalText, keyword, exactIndex)) {
    throw new Error("Keyword not found at specified position");
  }

  // 创建临时DOM元素
  const tempDiv: HTMLDivElement = document.createElement("div");
  tempDiv.innerHTML = formattedHtml;

  // 提取并创建文本位置映射
  let currentTextIndex: number = 0; // 用于跟踪当前文本位置
  let foundKeyword: boolean = false;

  // 遍历DOM并构建位置映射
  const traverseAndMap = (node: Node, textPath: Node[] = []): void => {
    if (foundKeyword) return;

    if (node.nodeType === Node.TEXT_NODE) {
      const text: string = node.textContent || "";

      // 查看当前索引是否与目标索引重叠
      if (
        exactIndex >= currentTextIndex &&
        exactIndex < currentTextIndex + text.length
      ) {
        // 精确计算在当前节点中的相对位置
        const relativeIndex: number = exactIndex - currentTextIndex;

        // 检查确认从该位置开始的子字符串是否匹配关键字
        if (
          text.substring(relativeIndex, relativeIndex + keyword.length) ===
          keyword
        ) {
          // 找到了精确匹配！

          // 分割文本节点
          const before: string = text.substring(0, relativeIndex);
          const matched: string = text.substring(
            relativeIndex,
            relativeIndex + keyword.length
          );
          const after: string = text.substring(relativeIndex + keyword.length);

          // 替换节点
          const parent: Node | null = node.parentNode;
          if (parent) {
            const beforeNode: Text = document.createTextNode(before);

            const highlightNode: HTMLElement = document.createElement("mask");
            highlightNode.id = highlightNodeId;
            highlightNode.setAttribute("style", generateMarkStyle({}));
            highlightNode.textContent = matched;

            const afterNode: Text = document.createTextNode(after);

            parent.insertBefore(beforeNode, node);
            parent.insertBefore(highlightNode, node);
            parent.insertBefore(afterNode, node);
            parent.removeChild(node);

            foundKeyword = true;
            return;
          }
        }
      }

      // 更新文本索引
      currentTextIndex += text.length;
    } else {
      // 处理元素节点
      const childTextPath: Node[] = [...textPath, node];
      for (let i = 0; i < node.childNodes.length && !foundKeyword; i++) {
        traverseAndMap(node.childNodes[i], childTextPath);
      }
    }
  };

  traverseAndMap(tempDiv);

  if (!foundKeyword) {
    throw new Error("Failed to find exact keyword position in formatted HTML");
  }

  return tempDiv.innerHTML;
};

// 初始化时保存原始JSON
watch(
  value,
  (newVal: string) => {
    if (newVal && !originalJsonText.value) {
      originalJsonText.value = escapeHtml(newVal);
    }
  },
  { immediate: true }
);

defineExpose({
  highlight
});
</script>

<style scoped></style>
