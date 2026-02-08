import React from 'react';

import type { RelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { type AnyLocation, SpecialLocation } from '../../select-location-types';
import { RelayLocationList } from '../relay-location-list';
import { SpecialLocationList } from '../special-location-list';

export interface CombinedLocationListProps<T> {
  relayLocations: AnyLocation[];
  specialLocations?: Array<SpecialLocation<T>>;
  allowAddToCustomList: boolean;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelectRelay: (value: RelayLocation) => void;
  onSelectSpecial: (value: T) => void;
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
export function CombinedLocationList<T>({
  specialLocations,
  onSelectSpecial,
  relayLocations,
  onSelectRelay,
  ...props
}: CombinedLocationListProps<T>) {
  return (
    <>
      {specialLocations !== undefined && specialLocations.length > 0 && (
        <SpecialLocationList {...props} source={specialLocations} onSelect={onSelectSpecial} />
      )}
      <RelayLocationList {...props} locations={relayLocations} onSelect={onSelectRelay} />
    </>
  );
}
