import { useState, useCallback, useLayoutEffect, RefCallback } from 'react';

export interface MeasureSize {
  width: number;
  height: number;
}

export function useMeasure<T extends Element = HTMLElement>(): [RefCallback<T>, MeasureSize] {
  const [size, setSize] = useState<MeasureSize>({ width: 0, height: 0 });
  const [node, setNode] = useState<T | null>(null);

  const ref: RefCallback<T> = useCallback((instance: T | null) => {
    setNode(instance);
  }, []);

  useLayoutEffect(() => {
    if (!node) return;

    const measure = () => {
      window.requestAnimationFrame(() => {
        const rect = node.getBoundingClientRect();
        setSize({ width: rect.width, height: rect.height });
      });
    };

    measure();

    const resizeObserver = new ResizeObserver(measure);
    resizeObserver.observe(node);

    return () => {
      resizeObserver.disconnect();
    };
  }, [node]);

  return [ref, size];
}
