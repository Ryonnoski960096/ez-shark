// 断点API

import { useIpc } from '@/hooks';
import { Breakpoint } from '@/hooks/useBreakpointConfig';
import { Response } from '../model';

const ipc = useIpc();

/**
 * 更新断点配置
 * @param breakpointList
 * @returns
 */
export function updateBreakpoint(
  breakpointList: Breakpoint[]
): Promise<Response> {
  return ipc.invoke('handle_debugger_command', {
    command: {
      type: 'update_breakpoint',
      breakpoints: breakpointList
    }
  });
}

/**
 * 删除断点
 * @param ids
 * @returns
 */
export function removeBreakpoint(ids: string[]): Promise<Response> {
  return ipc.invoke('handle_debugger_command', {
    command: {
      type: 'remove_breakpoint',
      ids
    }
  });
}
