import React from 'react';

import { ScrollToAnchorId } from '../../shared/ipc-types';
import { ListItemAnimation } from '../lib/components/list-item';
import { useHistory } from '../lib/history';
import { useScrollToReference } from '.';

export const useScrollToListItem = <T extends Element = HTMLDivElement>(
  id?: ScrollToAnchorId,
): {
  ref?: React.RefObject<T | null>;
  animation?: ListItemAnimation;
} => {
  const ref = React.useRef<T>(null);
  const history = useHistory();
  const { location } = history;
  const { state } = location;

  const anchorId = state?.options?.find((option) => option.type === 'scroll-to-anchor')?.id;
  const scroll = id === anchorId && anchorId !== undefined;

  const handleOnScrolled = React.useCallback(() => {
    history.replace(location, {
      ...state,
      options: state?.options?.filter((option) => option.type !== 'scroll-to-anchor'),
    });
  }, [history, location, state]);

  useScrollToReference(ref, scroll, handleOnScrolled);

  if (anchorId === undefined) {
    return {
      ref: undefined,
      animation: undefined,
    };
  }
  return {
    ref,
    animation: scroll ? 'flash' : 'dim',
  };
};
