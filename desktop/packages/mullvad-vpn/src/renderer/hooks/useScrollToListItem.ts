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

  const option = state?.options?.find((option) => option.type === 'scroll-to-anchor');

  const triggered = option?.triggered;
  const isMatchingId = option?.id === id;
  const scrollToReference = !triggered && isMatchingId;

  const handleOnScrolled = React.useCallback(() => {
    const newOptions = state?.options?.map((option) => {
      if (option.type === 'scroll-to-anchor') {
        return { ...option, triggered: true };
      }
      return option;
    });

    history.replace(location, {
      ...state,
      options: newOptions,
    });
  }, [history, location, state]);

  useFocusReference(ref, scrollToReference);
  useScrollToReference(ref, scrollToReference, handleOnScrolled);

  if (option === undefined) {
    return {
      ref: undefined,
      animation: undefined,
    };
  }

  const animation: ListItemAnimation | undefined = triggered
    ? undefined
    : isMatchingId
      ? 'flash'
      : 'dim';

  return {
    ref,
    animation,
  };
};
