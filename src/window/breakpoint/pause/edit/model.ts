export const requiredTabs = ["Header", "Cookie", "Text", "JSON Text", "Hex"];
export const optionalTabs = ["URL"];
export const allTabs = [...optionalTabs, ...requiredTabs];

export type RequiredTabType = (typeof requiredTabs)[number];
export type OptionalTabType = (typeof optionalTabs)[number];
export type TabType = RequiredTabType | OptionalTabType;

export const urlSelectOps = [
  {
    value: undefined,
    label: ""
  },
  {
    value: "GET",
    label: "GET"
  },
  {
    value: "POST",
    label: "POST"
  },
  {
    value: "PUT",
    label: "PUT"
  },
  {
    value: "DELETE",
    label: "DELETE"
  },
  {
    value: "HEAD",
    label: "HEAD"
  },
  {
    value: "OPTIONS",
    label: "OPTIONS"
  },
  {
    value: "CONNECT",
    label: "CONNECT"
  },
  {
    value: "TRACE",
    label: "TRACE"
  }
];

export const httpVersionOps = [
  {
    value: undefined,
    label: ""
  },
  {
    value: "HTTP/1.0",
    label: "HTTP/1.0"
  },
  {
    value: "HTTP/1.1",
    label: "HTTP/1.1"
  },
  {
    value: "HTTP/2.0",
    label: "HTTP/2.0"
  }
];
export type HttpMethodType = (typeof urlSelectOps)[number]["value"];
export type HttpVersionType = (typeof httpVersionOps)[number]["value"];

export interface urlData {
  method: HttpMethodType;
  url: string;
  params: Record<string, string>;
  httpVersion: HttpVersionType;
}

export interface DataItem {
  name: string;
  value: string;
}

export const tableOps = {
  stripe: true,
  sticky: true,
  border: true,
  size: "small" as "small",
  pagination: false as false,
  fit: true,
  flexible: false,
  "allow-drag-last-column": false,
  style: {
    width: "100%",
    height: "100%",
    "max-width": "100%"
  },
  "empty-text": ""
};

export interface TableDataType {
  value: any;
  text: any;
  record: any;
  index: number;
  renderIndex: number;
}

export interface CustomRenderParams<T> {
  text: string;
  record: T;
  index: number;
}

export interface trafficModificationAPIParams {
  id: string;
  modified_headers?: Record<string, string>;
  modified_body?: string;
  method: HttpMethodType;
  url: string;
}
