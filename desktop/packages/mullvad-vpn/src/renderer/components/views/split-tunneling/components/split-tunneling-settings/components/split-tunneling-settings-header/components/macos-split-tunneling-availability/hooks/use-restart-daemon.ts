import { useCallback } from 'react';

import { useAppContext } from '../../../../../../../../../../context';

export function useRestartDaemon() {
  const { daemonPrepareRestart } = useAppContext();

  const restartDaemon = useCallback(() => daemonPrepareRestart(true), [daemonPrepareRestart]);

  return restartDaemon;
}
