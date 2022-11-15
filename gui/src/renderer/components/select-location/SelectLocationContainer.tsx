import React, { useContext, useMemo, useRef, useState } from 'react';

import { Ownership, RelayLocation } from '../../../shared/daemon-rpc-types';
import { useNormalBridgeSettings, useNormalRelaySettings } from '../../lib/utilityHooks';
import { defaultExpandedLocations } from './select-location-helpers';
import { LocationType } from './select-location-types';
import SelectLocation from './SelectLocation';

type ExpandedLocations = Partial<Record<LocationType, Array<RelayLocation>>>;
type ScrollPosition = [number, number];

interface SelectLocationContext {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  activeFilter: boolean;
  expandedLocations: ExpandedLocations;
  setExpandedLocations: (
    arg: ExpandedLocations | ((prev: ExpandedLocations) => ExpandedLocations),
  ) => void;
  scrollPositions: React.RefObject<Partial<Record<LocationType, ScrollPosition>>>;
  selectedLocationRef: React.RefObject<HTMLDivElement>;
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

  const [expandedLocations, setExpandedLocations] = useState<
    Partial<Record<LocationType, Array<RelayLocation>>>
  >(() => defaultExpandedLocations(relaySettings, bridgeSettings));
  const scrollPositions = useRef<Partial<Record<LocationType, ScrollPosition>>>({});

  const ownershipActive = relaySettings !== undefined && relaySettings.ownership !== Ownership.any;
  const providersActive = relaySettings !== undefined && relaySettings.providers.length > 0;

  const value = useMemo(
    () => ({
      locationType,
      setLocationType,
      activeFilter: ownershipActive || providersActive,
      expandedLocations,
      setExpandedLocations,
      scrollPositions,
      selectedLocationRef,
    }),
    [locationType, relaySettings?.ownership, relaySettings?.providers, expandedLocations],
  );

  return (
    <selectLocationContext.Provider value={value}>
      <SelectLocation />
    </selectLocationContext.Provider>
  );
}
