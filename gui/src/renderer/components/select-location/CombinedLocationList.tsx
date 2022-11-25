import React from 'react';

import { RelayLocation } from '../../../shared/daemon-rpc-types';
import RelayLocationList from './RelayLocationList';
import {
  CountrySpecification,
  LocationList,
  LocationSelection,
  LocationSelectionType,
  SpecialLocation,
} from './select-location-types';
import SpecialLocationList from './SpecialLocationList';

export interface CombinedLocationListProps<T> {
  source: LocationList<T>;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: LocationSelection<T>) => void;
  onExpand: (location: RelayLocation) => void;
  onCollapse: (location: RelayLocation) => void;
  onWillExpand: (
    locationRect: DOMRect,
    expandedContentHeight: number,
    invokedByUser: boolean,
  ) => void;
  onTransitionEnd: () => void;
}

// Renders the special locations and the regular locations as separate lists
export default function CombinedLocationList<T>(props: CombinedLocationListProps<T>) {
  const specialLocations = props.source.filter(isSpecialLocation);
  const relayLocations = props.source.filter(isRelayLocation);

  return (
    <>
      <SpecialLocationList {...props} source={specialLocations} />
      <RelayLocationList {...props} source={relayLocations} />
    </>
  );
}

function isSpecialLocation<T>(
  location: CountrySpecification | SpecialLocation<T>,
): location is SpecialLocation<T> {
  return location.type === LocationSelectionType.special;
}

function isRelayLocation<T>(
  location: CountrySpecification | SpecialLocation<T>,
): location is CountrySpecification {
  return location.type === LocationSelectionType.relay;
}
