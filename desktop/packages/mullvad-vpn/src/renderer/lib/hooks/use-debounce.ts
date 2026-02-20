import React from 'react';

export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = React.useState<T>(value);
  const latestValueRef = React.useRef<T>(value);

  latestValueRef.current = value;

  React.useEffect(() => {
    if (delay <= 0) {
      setDebouncedValue(latestValueRef.current);
      return;
    }

    const timeoutId = setTimeout(() => {
      setDebouncedValue(latestValueRef.current);
    }, delay);

    return () => {
      clearTimeout(timeoutId);
    };
  }, [value, delay]);

  return debouncedValue;
}
