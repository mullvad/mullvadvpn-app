import { startApp, StartAppResponse } from '../utils';

export const startAppWithDaemon = async (): Promise<StartAppResponse> => {
  return startApp('.');
};
