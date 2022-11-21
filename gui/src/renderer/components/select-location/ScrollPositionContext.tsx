import React, { useCallback, useContext, useEffect, useMemo, useRef } from 'react';

import { useNormalRelaySettings } from '../../lib/utilityHooks';
import { CustomScrollbarsRef } from '../CustomScrollbars';
import { LocationType } from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';
import { SpacePreAllocationView } from './SpacePreAllocationView';

interface ScrollPositionContext {
  scrollPositions: React.RefObject<Partial<Record<LocationType, ScrollPosition>>>;
  selectedLocationRef: React.RefObject<HTMLDivElement>;
  scrollViewRef: React.RefObject<CustomScrollbarsRef>;
  spacePreAllocationViewRef: React.RefObject<SpacePreAllocationView>;
  saveScrollPosition: () => void;
  resetScrollPositions: () => void;
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

  const scrollViewRef = useRef<CustomScrollbarsRef>(null);
  const spacePreAllocationViewRef = useRef() as React.RefObject<SpacePreAllocationView>;
  const scrollPositions = useRef<Partial<Record<LocationType, ScrollPosition>>>({});
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

  const value = useMemo(
    () => ({
      scrollPositions,
      selectedLocationRef,
      scrollViewRef,
      spacePreAllocationViewRef,
      saveScrollPosition,
      resetScrollPositions,
    }),
    [],
  );

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
