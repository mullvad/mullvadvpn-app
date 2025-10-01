import React from 'react';

import { useFocusReferenceAfterPaint } from './useFocusReferenceAfterPaint';
import { useIsDefaultActiveElementAfterMount } from './useIsDefaultActiveElementAfterMount';

export const useInitialFocus = <T extends HTMLElement = HTMLDivElement>(): {
  ref?: React.RefObject<T | null>;
} => {
  const ref = React.useRef<T>(null);

  const isDefaultFocus = useIsDefaultActiveElementAfterMount();
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
