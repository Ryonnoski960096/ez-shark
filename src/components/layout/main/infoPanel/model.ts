import type { HttpRequestHeader } from "ant-design-vue/es/upload/interface";

export type TrafficStatus = "Complete";
export type Method =
  | "GET"
  | "POST"
  | "PUT"
  | "DELETE"
  | "HEAD"
  | "OPTIONS"
  | "CONNECT"
  | "TRACE"
  | "PATCH";

export interface Request {
  RequestHeaders: HttpRequestHeader;
  RequestBody: object;
}

export interface Response {
  ResponseHeaders: HttpRequestHeader;
  ResponseBody: object;
}
