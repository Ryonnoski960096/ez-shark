type JustifyType = "c" | "s" | "b" | "r" | "l" | "t" | "e";
type DirectionType = "row" | "col";
type AlignType = "start" | "center" | "end" | "between" | "around";
type WrapType = "n" | "w";

const justifyMap: Record<JustifyType, string> = {
  c: "center",
  s: "flex-start",
  b: "space-between",
  r: "flex-end",
  l: "flex-start",
  t: "center",
  e: "center",
};

const alignMap: Record<JustifyType, string> = {
  c: "center",
  s: "flex-start",
  b: "center",
  r: "center",
  l: "center",
  t: "flex-start",
  e: "flex-end",
};

const flexRules = [
  // 基础布局规则
  [
    /^f-([csbrlte])$/,
    ([, type]: [string, JustifyType]) => {
      return {
        display: "flex",
        "justify-content": justifyMap[type],
        "align-items": alignMap[type],
        "text-align": "center",
      };
    },
  ],
  // 方向规则
  [
    /^f-dir-([cr])$/,
    ([, dir]: [string, "c" | "r"]) => ({
      "flex-direction": dir === "c" ? "column" : "row",
    }),
  ],
  // 更复杂的 Flex 规则
  [
    /^f-(row|col)-(start|center|end|between|around)-(start|center|end)?$/,
    ([, dir, justify, align]: [string, DirectionType, AlignType, AlignType | undefined]) => {
      const dirMap = {
        row: "row",
        col: "column",
      };

      const justifyMap = {
        start: "flex-start",
        center: "center",
        end: "flex-end",
        between: "space-between",
        around: "space-around",
      };

      const alignMap = {
        start: "flex-start",
        center: "center",
        end: "flex-end",
      } as Record<AlignType, string>;

      return {
        display: "flex",
        "flex-direction": dirMap[dir],
        "justify-content": justifyMap[justify],
        "align-items": align ? alignMap[align] : "center",
      };
    },
  ],
  // 换行规则
  [
    /^f-wrap-([nw])$/,
    ([, type]: [string, WrapType]) => ({
      "flex-wrap": type === "n" ? "nowrap" : "wrap",
    }),
  ],
  // 间距规则
  [
    /^f-g-(\d+)$/,
    ([, gap]: [string, string]) => ({
      gap: `${gap}px`,
    }),
  ],
];

export default flexRules;
