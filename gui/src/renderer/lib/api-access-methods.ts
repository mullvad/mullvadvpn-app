import { useCallback, useRef, useState } from 'react';

import { CustomProxy } from '../../shared/daemon-rpc-types';
import { useScheduler } from '../../shared/scheduler';
import { useAppContext } from '../context';
import { useBoolean } from './utilityHooks';

// Each test needs to have an id to be cancelable.
let testId = 0;
function getTestId() {
  return ++testId;
}

export function useApiAccessMethodTest(
  autoReset = true,
  minDuration = 0,
): [
  boolean,
  boolean | undefined,
  (method: CustomProxy | string) => Promise<boolean | void>,
  () => void,
] {
  const { testApiAccessMethodById, testCustomApiAccessMethod } = useAppContext();
  const delayScheduler = useScheduler();

  // Whether or not the method is currently being tested.
  const [testing, setTesting, unsetTesting] = useBoolean();
  const testId = useRef<number>();
  const [testResult, setTestResult] = useState<boolean>();

  // A few seconds after the test has finished the result should not be displayed anymore. This
  // scheduler is used to clear it.
  const testResultResetScheduler = useScheduler();

  const testApiAccessMethod = useCallback(async (method: CustomProxy | string) => {
    testResultResetScheduler.cancel();
    setTestResult(undefined);

    const id = getTestId();
    testId.current = id;

    setTesting();
    let reachable;

    const submitTimestamp = Date.now();
    try {
      if (typeof method === 'string') {
        reachable = await testApiAccessMethodById(method);
      } else {
        reachable = await testCustomApiAccessMethod(method);
      }
    } catch {
      reachable = false;
    }

    // Make sure the loading text is displayed for at least `minDuration` milliseconds.
    const submitDuration = Date.now() - submitTimestamp;
    if (submitDuration < minDuration) {
      await new Promise<void>((resolve) =>
        delayScheduler.schedule(resolve, minDuration - submitDuration),
      );
    }

    if (id !== testId.current) {
      return;
    }

    setTestResult(reachable);
    unsetTesting();

    if (autoReset) {
      testResultResetScheduler.schedule(() => setTestResult(undefined), 5000);
    }

    return reachable;
  }, []);

  const resetTestResult = useCallback(() => {
    testId.current = undefined;
    unsetTesting();
    setTestResult(undefined);
  }, []);

  return [testing, testResult, testApiAccessMethod, resetTestResult];
}
