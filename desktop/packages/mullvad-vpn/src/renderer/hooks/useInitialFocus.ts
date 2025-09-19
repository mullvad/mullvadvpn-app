import React from 'react';

import { useFocusReference } from './useFocusReference';
import { useIsDefaultFocusOnLoad } from './useIsDefaultFocusOnLoad';

export const useInitialFocus = <T extends HTMLElement = HTMLDivElement>(): {
  ref?: React.RefObject<T | null>;
} => {
  const ref = React.useRef<T>(null);

  const isDefaultFocus = useIsDefaultFocusOnLoad();

  useFocusReference(ref, isDefaultFocus);

  if (!isDefaultFocus)
    return {
      ref: undefined,
    };
  return {
    ref,
  };
};
