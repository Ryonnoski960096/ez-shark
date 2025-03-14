import { useIpc } from "@/hooks";
import { ITrafficDataDetail } from "@/stores/traffic";
import { trafficModificationAPIParams } from "@/window/breakpoint/pause/edit/model";
import { Response } from "../model";

const ipc = useIpc();

/**
 * 查询流量详情
 */
export function queryTrafficDetail(id: number) {
  return ipc.invoke<ITrafficDataDetail>("get_traffic_detail", {
    id
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
