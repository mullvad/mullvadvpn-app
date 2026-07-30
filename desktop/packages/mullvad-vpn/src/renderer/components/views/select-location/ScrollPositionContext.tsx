import React, { useCallback, useContext, useMemo, useRef } from 'react';

import { useRecents } from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { useDebounce } from '../../../lib/hooks/use-debounce';
import { useStyledRef } from '../../../lib/utility-hooks';
import { CustomScrollbarsRef } from '../../CustomScrollbars';
import { SpacePreAllocationView } from './components';

// Context containing the scroll position for each location type and methods to interact with it.
interface ScrollPositionContext {
  // The selected location element is used to scroll to it when opening the view
  selectedLocationRef: React.RefObject<HTMLDivElement | null>;
  // The scroll view container is used to get the current scroll position and to restore an old one
  scrollViewRef: React.RefObject<CustomScrollbarsRef | null>;
  // The space pre allocation view is used to enable smooth scrolling when opening locations
  spacePreAllocationViewRef: React.RefObject<SpacePreAllocationView | null>;
  scrollIntoView: (rect: DOMRect) => void;
  resetHeight: () => void;
  scrollTop: number;
  setScrollTop: (value: number) => void;
  resetScroll: () => void;
}

type ScrollPosition = [number, number];

const scrollPositionContext = React.createContext<ScrollPositionContext | undefined>(undefined);

export function useScrollPositionContext() {
  return useContext(scrollPositionContext)!;
}

interface ScrollPositionContextProps {
  children: React.ReactNode;
}

export function ScrollPositionContextProvider(props: ScrollPositionContextProps) {
  const { hasRecents } = useRecents();

  const scrollPositions = useRef<Partial<Record<LocationType, ScrollPosition>>>({});
  const scrollViewRef = useRef<CustomScrollbarsRef>(null);
  const spacePreAllocationViewRef = useStyledRef<SpacePreAllocationView>();
  const selectedLocationRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = React.useState(0);
  const debouncedScrollTop = useDebounce(scrollTop, 50);

  const scrollIntoView = useCallback((rect: DOMRect) => {
    scrollViewRef.current?.scrollIntoView(rect);
  }, []);

  const resetHeight = useCallback(
    () => spacePreAllocationViewRef.current?.reset(),
    [spacePreAllocationViewRef],
  );

  const resetScroll = useCallback(() => {
    if (hasRecents) {
      // Scroll to top if there are recents.
      scrollViewRef.current?.scrollToTop();
      return;
    } else {
      // Scroll to the selected location if there are no recents.
      if (selectedLocationRef.current) {
        scrollViewRef.current?.scrollToElement(selectedLocationRef.current, 'middle');
      } else {
        scrollViewRef.current?.scrollToTop();
      }
    }
  }, [hasRecents]);

  const value = useMemo(
    () => ({
      scrollPositions,
      selectedLocationRef,
      scrollViewRef,
      spacePreAllocationViewRef,
      scrollIntoView,
      resetHeight,
      scrollTop: debouncedScrollTop,
      setScrollTop,
      resetScroll,
    }),
    [spacePreAllocationViewRef, scrollIntoView, resetHeight, debouncedScrollTop, resetScroll],
  );

  return (
    <scrollPositionContext.Provider value={value}>{props.children}</scrollPositionContext.Provider>
  );
}
