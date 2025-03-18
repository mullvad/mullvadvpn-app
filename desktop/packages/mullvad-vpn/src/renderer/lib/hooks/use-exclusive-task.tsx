import { useCallback, useRef, useState } from 'react';

export const useExclusiveTask = (task: () => Promise<void>) => {
  const isRunningRef = useRef(false);
  const [isRunning, setIsRunning] = useState(false);

  const runTask = useCallback(async () => {
    if (isRunningRef.current) {
      return;
    }
    isRunningRef.current = true;
    setIsRunning(true);
    try {
      await task();
    } finally {
      isRunningRef.current = false;
      setIsRunning(false);
    }
  }, [task]);

  return [runTask, isRunning] as const;
};
