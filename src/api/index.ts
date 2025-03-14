import { Response } from './model';

/**
 * 接口是否返回成功
 */
export function isSuccess(res: string | Response): boolean {
  return res === 'Success';
}
