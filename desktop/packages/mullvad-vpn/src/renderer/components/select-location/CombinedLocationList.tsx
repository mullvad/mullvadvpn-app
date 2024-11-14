import React from 'react';

import { RelayLocation } from '../../../shared/daemon-rpc-types';
import RelayLocationList from './RelayLocationList';
import { RelayList, SpecialLocation } from './select-location-types';
import SpecialLocationList from './SpecialLocationList';

export interface CombinedLocationListProps<T> {
  relayLocations: RelayList;
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
export default function CombinedLocationList<T>(props: CombinedLocationListProps<T>) {
  return (
    <>
      {props.specialLocations !== undefined && props.specialLocations.length > 0 && (
        <SpecialLocationList
          {...props}
          source={props.specialLocations}
          onSelect={props.onSelectSpecial}
        />
      )}
      <RelayLocationList {...props} source={props.relayLocations} onSelect={props.onSelectRelay} />
    </>
  );
}
