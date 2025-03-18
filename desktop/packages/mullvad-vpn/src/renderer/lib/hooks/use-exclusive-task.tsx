import React from 'react';
import { useCallback, useState } from 'react';

export const useExclusiveTask = (task: () => Promise<void>) => {
  const [running, setRunning] = useState(false);

  const run = useCallback(async (): Promise<void | undefined> => {
    if (running) {
      return;
    }
    setRunning(true);
    try {
      await task();
    } finally {
      setRunning(false);
    }
  }, [task, running]);

  const result = React.useMemo(() => [run, running] as const, [run, running]);

  return result;
};
