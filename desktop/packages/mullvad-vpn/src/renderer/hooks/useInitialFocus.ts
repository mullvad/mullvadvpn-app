import React from 'react';

import { useFocusReferenceAfterPaint } from './useFocusReferenceAfterPaint';
import { useIsDefaultActiveElementAfterPaint } from './useIsDefaultActiveElementAfterPaint';

export const useInitialFocus = <T extends HTMLElement = HTMLDivElement>(): {
  ref?: React.RefObject<T | null>;
} => {
  const ref = React.useRef<T>(null);

  const isDefaultFocus = useIsDefaultActiveElementAfterPaint();
  const shouldFocus = isDefaultFocus === true;

  useFocusReferenceAfterPaint(ref, shouldFocus);

  if (!isDefaultFocus)
    return {
      ref: undefined,
    };
  return {
    ref,
  };
};
