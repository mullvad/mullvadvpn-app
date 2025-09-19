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
  const history = useHistory();
  const { location } = history;
  const { state } = location;

  const scrollToAnchorOption = state?.options?.find((option) => option.type === 'scroll-to-anchor');
  const shouldScroll = scrollToAnchorOption && scrollToAnchorOption.id === id;

  const handleOnScrolled = React.useCallback(() => {
    history.replace(location, {
      ...state,
      options: state?.options?.filter((option) => option.type !== 'scroll-to-anchor'),
    });
  }, [history, location, state]);

  useScrollToReference(ref, shouldScroll, handleOnScrolled);
  useFocusReference(ref, shouldScroll);

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
