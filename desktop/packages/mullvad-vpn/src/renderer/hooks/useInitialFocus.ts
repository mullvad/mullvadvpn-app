import React from 'react';

import { useHistory } from '../lib/history';
import { useFocusReference } from './useFocusReference';

export const useInitialFocus = <T extends HTMLElement = HTMLDivElement>(
  focus?: boolean,
): {
  ref?: React.RefObject<T | null>;
} => {
  const ref = React.useRef<T>(null);
  const { location } = useHistory();
  const { state } = location;

  // Should not focus if we have requested to scroll to an anchor.
  const shouldScrollToAnchor = state?.options?.find((option) => option.type === 'scroll-to-anchor')
    ? true
    : false;

  const shouldFocus = focus && !shouldScrollToAnchor;

  useFocusReference(ref, shouldFocus);

  if (!shouldFocus)
    return {
      ref: undefined,
    };
  return {
    ref,
  };
};
