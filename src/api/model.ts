/**
 * 接口返回
 */
export type Response<T = any> = "Success" | ("Fail" | T);

export type Status = "Success" | "Fail";

/**
 * 后端 payload
 */
export type Payload<T = any> = {
  data: T;
  message: string;
  status: Status;
};
