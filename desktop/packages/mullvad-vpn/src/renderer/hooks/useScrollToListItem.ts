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

  const handleScrolled = React.useCallback(() => {
    const options = state?.options?.filter((option) => {
      if (option.type === 'scroll-to-anchor') {
        return option.id !== anchorId;
      }

      return true;
    });

    history.replace(location, {
      ...state,
      options,
    });
  }, [anchorId, history, location, state]);

  useScrollToReference(ref, scroll, handleScrolled);

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
