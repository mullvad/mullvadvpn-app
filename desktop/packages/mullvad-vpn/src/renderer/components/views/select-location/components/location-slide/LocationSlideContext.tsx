import React from 'react';

import { useRecents } from '../../../../../features/locations/hooks';
import { useDebounce } from '../../../../../lib/hooks/use-debounce';
import { useStyledRef } from '../../../../../lib/utility-hooks';
import { CustomScrollbarsRef } from '../../../../CustomScrollbars';
import { SpacePreAllocationView } from '..';

// Context containing the scroll position for each location type and methods to interact with it.
type LocationSlideContextProps = Omit<LocationSlideContextProviderProps, 'children'> & {
  // The selected location element is used to scroll to it when opening the view
  selectedLocationRef: React.RefObject<HTMLDivElement | null>;
  // The scroll view container is used to get the current scroll position and to restore an old one
  scrollViewRef: React.RefObject<CustomScrollbarsRef | null>;
  // The space pre allocation view is used to enable smooth scrolling when opening locations
  spacePreAllocationViewRef: React.RefObject<SpacePreAllocationView | null>;
  scrollIntoView: (rect: DOMRect) => void;
  resetHeight: () => void;
  resetScroll: () => void;
};

const LocationSlideContext = React.createContext<LocationSlideContextProps | undefined>(undefined);

export function useLocationSlideContext() {
  const context = React.useContext(LocationSlideContext);
  if (!context) {
    throw new Error('useLocationSlideContext must be used within a LocationSlideContextProvider');
  }
  return context;
}

type LocationSlideContextProviderProps = React.PropsWithChildren;

export function LocationSlideContextProvider(props: LocationSlideContextProviderProps) {
  const { hasRecents } = useRecents();

  const scrollViewRef = React.useRef<CustomScrollbarsRef>(null);
  const spacePreAllocationViewRef = useStyledRef<SpacePreAllocationView>();
  const selectedLocationRef = React.useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = React.useState(0);
  const debouncedScrollTop = useDebounce(scrollTop, 50);

  const scrollIntoView = React.useCallback((rect: DOMRect) => {
    scrollViewRef.current?.scrollIntoView(rect);
  }, []);

  const resetHeight = React.useCallback(
    () => spacePreAllocationViewRef.current?.reset(),
    [spacePreAllocationViewRef],
  );

  const resetScroll = React.useCallback(() => {
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

  const value = React.useMemo(
    () => ({
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
    <LocationSlideContext.Provider value={value}>{props.children}</LocationSlideContext.Provider>
  );
}
