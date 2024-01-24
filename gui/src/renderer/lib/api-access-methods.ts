import { useCallback, useRef, useState } from 'react';

import { CustomProxy } from '../../shared/daemon-rpc-types';
import { useScheduler } from '../../shared/scheduler';
import { useAppContext } from '../context';
import { useBoolean } from './utilityHooks';

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
  const [testResult, setTestResult] = useState<boolean>();
  // We keep the promise for the most recent test to compare it when we receive the results to know
  // if it's canceled or not.
  const lastTestPromise = useRef<Promise<boolean>>();

  // A few seconds after the test has finished the result should not be displayed anymore. This
  // scheduler is used to clear it.
  const testResultResetScheduler = useScheduler();

  const testApiAccessMethod = useCallback(async (method: CustomProxy | string) => {
    testResultResetScheduler.cancel();
    setTestResult(undefined);

    setTesting();
    let reachable;
    let testPromise;

    const submitTimestamp = Date.now();
    try {
      testPromise =
        typeof method === 'string'
          ? testApiAccessMethodById(method)
          : testCustomApiAccessMethod(method);

      lastTestPromise.current = testPromise;
      reachable = await testPromise;
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

    if (testPromise !== lastTestPromise.current) {
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
    lastTestPromise.current = undefined;
    unsetTesting();
    setTestResult(undefined);
  }, []);

  return [testing, testResult, testApiAccessMethod, resetTestResult];
}
