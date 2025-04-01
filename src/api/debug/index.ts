// 断点API

import { useIpc } from "@/hooks";
import type { Response } from "../model";

const ipc = useIpc();

/**
 * 获取所有 traffic
 * @returns
 */
export function getTraffics(): Promise<Response> {
  return ipc.invoke("get_traffics");
}
