import React, { useContext, useMemo, useRef, useState } from 'react';

import { RelayLocation } from '../../../shared/daemon-rpc-types';
import { useNormalBridgeSettings, useNormalRelaySettings } from '../../lib/utilityHooks';
import {CustomScrollbarsRef} from '../CustomScrollbars';
import { defaultExpandedLocations } from './select-location-helpers';
import { LocationType } from './select-location-types';
import SelectLocation from './SelectLocation';
import {SpacePreAllocationView} from './SpacePreAllocationView';

type ExpandedLocations = Partial<Record<LocationType, Array<RelayLocation>>>;
type ScrollPosition = [number, number];

interface SelectLocationContext {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
  expandedLocations: ExpandedLocations;
  setExpandedLocations: (
    arg: ExpandedLocations | ((prev: ExpandedLocations) => ExpandedLocations),
  ) => void;
  scrollPositions: React.RefObject<Partial<Record<LocationType, ScrollPosition>>>;
  selectedLocationRef: React.RefObject<HTMLDivElement>;
  scrollViewRef: React.RefObject<CustomScrollbarsRef>;
  spacePreAllocationViewRef: React.RefObject<SpacePreAllocationView>;
}

const selectLocationContext = React.createContext<SelectLocationContext | undefined>(undefined);

export function useSelectLocationContext() {
  return useContext(selectLocationContext)!;
}

export default function SelectLocationContainer() {
  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useNormalBridgeSettings();
  const [locationType, setLocationType] = useState(LocationType.exit);

  const selectedLocationRef = useRef<HTMLDivElement>(null);
  const scrollViewRef = useRef<CustomScrollbarsRef>(null);
  const spacePreAllocationViewRef = useRef() as React.RefObject<SpacePreAllocationView>;

  const [expandedLocations, setExpandedLocations] = useState<
    Partial<Record<LocationType, Array<RelayLocation>>>
  >(() => defaultExpandedLocations(relaySettings, bridgeSettings));
  const scrollPositions = useRef<Partial<Record<LocationType, ScrollPosition>>>({});

  const [searchTerm, setSearchTerm] = useState('');

  const value = useMemo(
    () => ({
      locationType,
      setLocationType,
      searchTerm,
      setSearchTerm,
      expandedLocations,
      setExpandedLocations,
      scrollPositions,
      selectedLocationRef,
      scrollViewRef,
      spacePreAllocationViewRef,
    }),
    [
      locationType,
      relaySettings?.ownership,
      relaySettings?.providers,
      expandedLocations,
      searchTerm,
    ],
  );

  return (
    <selectLocationContext.Provider value={value}>
      <SelectLocation />
    </selectLocationContext.Provider>
  );
}
