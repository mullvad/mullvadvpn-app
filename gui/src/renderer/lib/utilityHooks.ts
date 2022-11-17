import React, { useCallback, useEffect, useRef, useState } from 'react';

import { useSelector } from '../redux/store';

export function useMounted() {
  const mountedRef = useRef(false);
  const isMounted = useCallback(() => mountedRef.current, []);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  return isMounted;
}

export function useCombinedRefs<T>(...refs: (React.Ref<T> | undefined)[]): React.RefCallback<T> {
  return useCallback((element: T | null) => refs.forEach((ref) => assignToRef(element, ref)), []);
}

export function assignToRef<T>(element: T | null, ref?: React.Ref<T>) {
  if (typeof ref === 'function') {
    ref(element);
  } else if (ref && element) {
    (ref as React.MutableRefObject<T>).current = element;
  }
}

export function useAsyncEffect(
  effect: () => Promise<void | (() => void | Promise<void>)>,
  dependencies: unknown[],
): void {
  const isMounted = useMounted();

  useEffect(() => {
    const promise = effect();
    return () => {
      void promise.then((destructor) => {
        if (isMounted() && destructor) {
          return destructor();
        }
      });
    };
  }, dependencies);
}

export function useBoolean(initialValue = false) {
  const [value, setValue] = useState(initialValue);

  const setTrue = useCallback(() => setValue(true), []);
  const setFalse = useCallback(() => setValue(false), []);
  const toggle = useCallback(() => setValue((value) => !value), []);

  return [value, setTrue, setFalse, toggle] as const;
}

export function useNormalRelaySettings() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  return 'normal' in relaySettings ? relaySettings.normal : undefined;
}

export function useNormalBridgeSettings() {
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);
  return 'normal' in bridgeSettings ? bridgeSettings.normal : undefined;
}

const sharedMemoData: Record<
  string,
  { value: unknown; dependencies: Array<unknown> | undefined }
> = {};
export function useSharedMemo<T>(
  key: string,
  factory: () => T,
  dependencies: Array<unknown> | undefined,
): T {
  const data = sharedMemoData[key];
  if (
    data === undefined ||
    data.dependencies === undefined ||
    dependencies === undefined ||
    data.dependencies.length !== dependencies.length ||
    data.dependencies.some((item, i) => item !== dependencies[i])
  ) {
    const value = factory();
    sharedMemoData[key] = { value, dependencies };
    return value;
  } else {
    return data.value as T;
  }
}
