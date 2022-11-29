import React, { useCallback, useContext, useEffect, useMemo, useRef } from 'react';

import { useNormalRelaySettings } from '../../lib/utilityHooks';
import { CustomScrollbarsRef } from '../CustomScrollbars';
import { LocationType } from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';
import { SpacePreAllocationView } from './SpacePreAllocationView';

// Context containing the scroll position for each location type and methods to interact with it.
interface ScrollPositionContext {
  scrollPositions: React.RefObject<Partial<Record<LocationType, ScrollPosition>>>;
  // The selected location element is used to scroll to it when opening the view
  selectedLocationRef: React.RefObject<HTMLDivElement>;
  // The scroll view container is used to get the current scroll position and to restore an old one
  scrollViewRef: React.RefObject<CustomScrollbarsRef>;
  // The space pre allocation view is used to enable smooth scrolling when opening locations
  spacePreAllocationViewRef: React.RefObject<SpacePreAllocationView>;
  saveScrollPosition: () => void;
  resetScrollPositions: () => void;
  scrollIntoView: (rect: DOMRect) => void;
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
  const { locationType, searchTerm } = useSelectLocationContext();
  const relaySettings = useNormalRelaySettings();

  const scrollPositions = useRef<Partial<Record<LocationType, ScrollPosition>>>({});
  const scrollViewRef = useRef<CustomScrollbarsRef>(null);
  const spacePreAllocationViewRef = useRef() as React.RefObject<SpacePreAllocationView>;
  const selectedLocationRef = useRef<HTMLDivElement>(null);

  const saveScrollPosition = useCallback(() => {
    const scrollPosition = scrollViewRef.current?.getScrollPosition();
    if (scrollPositions.current && scrollPosition) {
      scrollPositions.current[locationType] = scrollPosition;
    }
  }, [locationType]);

  const resetScrollPositions = useCallback(() => {
    for (const locationTypeVariant of [LocationType.entry, LocationType.exit]) {
      if (
        scrollPositions.current &&
        (scrollPositions.current[locationTypeVariant] || locationTypeVariant === locationType)
      ) {
        scrollPositions.current[locationTypeVariant] = [0, 0];
      }
    }
  }, [locationType]);

  const scrollIntoView = useCallback((rect: DOMRect) => {
    scrollViewRef.current?.scrollIntoView(rect);
  }, []);

  const value = useMemo(
    () => ({
      scrollPositions,
      selectedLocationRef,
      scrollViewRef,
      spacePreAllocationViewRef,
      saveScrollPosition,
      resetScrollPositions,
      scrollIntoView,
    }),
    [saveScrollPosition, resetScrollPositions],
  );

  // Restore the scroll position when parameters change
  useEffect(() => {
    const scrollPosition = scrollPositions.current?.[locationType];
    if (scrollPosition) {
      scrollViewRef.current?.scrollTo(...scrollPosition);
    } else if (selectedLocationRef.current) {
      scrollViewRef.current?.scrollToElement(selectedLocationRef.current, 'middle');
    } else {
      scrollViewRef.current?.scrollToTop();
    }
  }, [locationType, searchTerm, relaySettings?.ownership, relaySettings?.providers]);

  return (
    <scrollPositionContext.Provider value={value}>{props.children}</scrollPositionContext.Provider>
  );
}
