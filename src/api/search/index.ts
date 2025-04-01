import { useIpc } from "@/hooks";
import type { SearchQuery, SearchResult } from "./model";

const ipc = useIpc();

export async function search(data: SearchQuery, sessionId: string) {
  return ipc.invoke<SearchResult>("search", {
    data,
    sessionId
  });
}
