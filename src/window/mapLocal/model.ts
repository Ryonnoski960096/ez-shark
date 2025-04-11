export interface MapLocalItem {
  id?: string;
  enabled: boolean;
  url: string;
  headerLocal: string;
  bodyLocal: string;
}

/**
 * @description: 本地文件映射
 * @param key hashId
 */
export interface MapLocal {
  toolEnabled: boolean;
  mapLocals: Record<string, MapLocalItem>;
}

export enum MapLocalEvent {
  SUBMIT = "submit"
}
