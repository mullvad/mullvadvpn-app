import React from 'react';

import { ScrollToAnchorId } from '../../shared/ipc-types';
import { ListItemAnimation } from '../lib/components/list-item';
import { useHistory } from '../lib/history';
import { useFocusReference } from './useFocusReference';
import { useScrollToReference } from './useScrollToReference';

export const useScrollToListItem = <T extends HTMLElement = HTMLDivElement>(
  id?: ScrollToAnchorId,
): {
  ref?: React.RefObject<T | null>;
  animation?: ListItemAnimation;
} => {
  const ref = React.useRef<T>(null);
  const { location, action } = useHistory();
  const { state } = location;

  const isPop = action === 'POP';
  const anchorId = state?.options?.find((option) => option.type === 'scroll-to-anchor')?.id;
  const scroll = id === anchorId && !isPop;
  useScrollToReference(ref, scroll);
  useFocusReference(ref, scroll);

  if (anchorId === undefined || isPop)
    return {
      ref: undefined,
      animation: undefined,
    };
  return {
    ref,
    animation: scroll ? 'flash' : 'dim',
  };
};
