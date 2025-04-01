import { useIpc } from "@/hooks";
import type { ITrafficDataDetail } from "@/stores/traffic";
import type { trafficModificationAPIParams } from "@/components/contents/model";
import type { Response } from "../model";
import { useSessionStore } from "@/stores/session";

const ipc = useIpc();

/**
 * 查询流量详情
 */
export function queryTrafficDetail(id: number) {
  const sessionStore = useSessionStore();
  return ipc.invoke<ITrafficDataDetail>("get_traffic_detail", {
    id,
    sessionId: sessionStore.currentSession ?? ""
  });
}

/**
 * 修改流量
 * @param modified_type
 * @param data
 * @returns
 */
export function trafficModification(
  modified_type: "request" | "response",
  data: trafficModificationAPIParams
): Promise<Response> {
  return ipc.invoke("handle_debugger_command", {
    command: {
      type: "traffic_modification",
      modified_type,
      ...data
    }
  });
}

export function trafficContinue(id: string): Promise<Response> {
  return ipc.invoke("handle_debugger_command", {
    command: {
      type: "continue",
      id
    }
  });
}

export function resend(id: number): Promise<Response> {
  return ipc.invoke("resend", {
    id
  });
}

export function onResend(
  data: trafficModificationAPIParams
): Promise<Response> {
  return ipc.invoke("on_resend", {
    data: {
      modified_type: "resend",
      ...data
    }
  });
}

// 删除流量
export function deleteTraffic(ids: number[]) {
  const sessionStore = useSessionStore();
  return ipc.invoke("delete_traffic", {
    ids,
    sessionId: sessionStore.currentSession ?? ""
  });
}
