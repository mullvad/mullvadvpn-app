import React, {
  useCallback,
  useEffect,
  useInsertionEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

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

export function useStyledRef<T>(): React.MutableRefObject<T> {
  return useRef() as React.MutableRefObject<T>;
}

export function useCombinedRefs<T>(...refs: (React.Ref<T> | undefined)[]): React.RefCallback<T> {
  return useMemo(
    () => (element: T | null) => refs.forEach((ref) => assignToRef(element, ref)),
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [...refs],
  );
}

export function assignToRef<T>(element: T | null, ref?: React.Ref<T>) {
  if (typeof ref === 'function') {
    ref(element);
  } else if (ref && element) {
    (ref as React.MutableRefObject<T>).current = element;
  }
}

export function useBoolean(initialValue = false) {
  const [value, setValue] = useState(initialValue);

  const setTrue = useCallback(() => setValue(true), []);
  const setFalse = useCallback(() => setValue(false), []);
  const toggle = useCallback(() => setValue((value) => !value), []);

  return [value, setTrue, setFalse, toggle] as const;
}

// This hook returns a function that can be used to force a rerender of a component, and
// additionally also returns a variable that can be used to trigger effects as a result. This is a
// hack and should be avoided unless there are no better ways.
export function useRerenderer(): [() => void, number] {
  const [count, setCount] = useState(0);
  const rerender = useCallback(() => setCount((count) => count + 1), []);
  return [rerender, count];
}

function calculateInitialValue<T>(initialValue: (() => T) | T): T {
  if (typeof initialValue === 'function') {
    const getInitialValue = initialValue as () => T;
    return getInitialValue();
  } else {
    return initialValue;
  }
}

export function useInitialValue<T>(initialValue: (() => T) | T): T {
  const [value] = useState(calculateInitialValue(initialValue));
  return value;
}

type Fn<T extends unknown[], R> = (...args: T) => R;

export function useEffectEvent<Args extends unknown[]>(
  fn: Fn<Args, void | undefined | Promise<void | undefined>>,
): Fn<Args, void> {
  const ref = useRef<Fn<Args, void>>(fn);

  useInsertionEffect(() => {
    ref.current = fn;
  }, [fn]);

  return useCallback((...args: Args) => ref.current(...args), []);
}

// Alias for useEffectEvent, but with another name since the effect event is named after a very
// specific usecase.
export const useRefCallback = useEffectEvent;

export function useLastDefinedValue<T>(value: T): T {
  const [definedValue, setDefinedValue] = useState(value);

  useEffect(() => setDefinedValue((prev) => value ?? prev), [value]);

  return value ?? definedValue;
}
