import React from 'react';

import { ScrollToAnchorId } from '../../shared/ipc-types';
import { ListItemAnimation } from '../lib/components/list-item';
import { useHistory } from '../lib/history';
import { useFocusReferenceBeforePaint } from './useFocusReferenceBeforePaint';
import { useScrollToReference } from './useScrollToReference';

export const useScrollToListItem = <T extends HTMLElement = HTMLDivElement>(
  id?: ScrollToAnchorId,
): {
  ref?: React.RefObject<T | null>;
  animation?: ListItemAnimation;
} => {
  const ref = React.useRef<T>(null);
  const history = useHistory();
  const { location } = history;
  const { state } = location;

  const scrollToAnchorOption = state?.options?.find((option) => option.type === 'scroll-to-anchor');
  const shouldScroll = scrollToAnchorOption && scrollToAnchorOption.id === id;

  const handleScrolled = React.useCallback(() => {
    const options = state?.options?.filter((option) => {
      if (option.type === 'scroll-to-anchor') {
        return option.id !== scrollToAnchorOption?.id;
      }

      return true;
    });

    history.replace(location, {
      ...state,
      options,
    });
  }, [history, location, scrollToAnchorOption?.id, state]);

  useScrollToReference(ref, shouldScroll, handleScrolled);
  useFocusReferenceBeforePaint(ref, shouldScroll);

  if (scrollToAnchorOption === undefined) {
    return {
      ref: undefined,
      animation: undefined,
    };
  }

  return {
    ref,
    animation: shouldScroll ? 'flash' : 'dim',
  };
};
