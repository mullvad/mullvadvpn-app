import React, { useCallback, useEffect, useRef, useState } from 'react';

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

export function useBoolean(initialValue: boolean) {
  const [value, setValue] = useState(initialValue);

  const setTrue = useCallback(() => setValue(true), []);
  const setFalse = useCallback(() => setValue(false), []);
  const toggle = useCallback(() => setValue((value) => !value), []);

  return [value, setTrue, setFalse, toggle] as const;
}
