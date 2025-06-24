import { MockedTestUtils } from '../mocked-utils';

export const createIpc = (util: MockedTestUtils) => {
  const createMockResponse = <T>(channel: string, response: T) =>
    util.sendMockIpcResponse<T>({
      channel,
      response,
    });

  return {
    handle: {},
    send: {
      daemonDisconnected: () => createMockResponse('daemon-disconnected', {}),
      daemonAllowed: (allowed: boolean) => createMockResponse('daemon-daemonAllowed', allowed),
    },
  };
};
