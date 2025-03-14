import { RBody } from "@/stores/traffic";
import { HttpRequestHeader } from "ant-design-vue/es/upload/interface";

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

export interface OverviewData {
  Protocol: string;
  Url: string;
  Status: TrafficStatus | "暂无";
  ResponseCode: number;
  Method: Method;
  Request: Request;
  Response: Response;
  Body?: RBody;
}

export type HTTPMeta = OverviewData | Request | Response;
