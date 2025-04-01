import type { MenuItem } from "@/components/dropdownMenu/model";
import { useExport } from "@/hooks";
import { commonIE } from "@/utils/tools";

export const createExportMenuItems = (sessionId: string): MenuItem[] => {
  if (!sessionId || sessionId === "") return [];

  const exportTool = useExport(sessionId);
  return [
    {
      label: "Export as all Markdown",
      action: "markdown",
      click: () => commonIE(exportTool.exportMarkdown)
    },
    {
      label: "Export as all cURL",
      action: "curl",
      click: () => commonIE(exportTool.exportCurl)
    },
    {
      label: "Export as all HAR",
      action: "har",
      click: () => commonIE(exportTool.exportHar)
    },
    {
      label: "Export as all JSON",
      action: "json",
      click: () => commonIE(exportTool.exportJson)
    },
    {
      label: "Export as Session",
      action: "session",
      click: () => commonIE(exportTool.exportJson)
    }
  ];
};
