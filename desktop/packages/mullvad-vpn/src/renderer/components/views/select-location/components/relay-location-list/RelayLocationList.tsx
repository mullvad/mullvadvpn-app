import React from 'react';

import { type RelayLocation as RelayLocationType } from '../../../../../../shared/daemon-rpc-types';
import * as Cell from '../../../../cell';
import { RelayList } from '../../select-location-types';
import { RelayLocation } from '../relay-location';

interface CommonProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  allowAddToCustomList: boolean;
  onSelect: (value: RelayLocationType) => void;
  onExpand: (location: RelayLocationType) => void;
  onCollapse: (location: RelayLocationType) => void;
  onWillExpand: (
    locationRect: DOMRect,
    expandedContentHeight: number,
    invokedByUser: boolean,
  ) => void;
  onTransitionEnd: () => void;
}

interface RelayLocationsProps extends CommonProps {
  source: RelayList;
}

export function RelayLocationList({ source, ...props }: RelayLocationsProps) {
  return (
    <Cell.Group $noMarginBottom>
      {source.map((country) => (
        <RelayLocation
          key={getLocationKey(country.location)}
          source={country}
          level={0}
          {...props}
        />
      ))}
    </Cell.Group>
  );
}

function getLocationKey(location: RelayLocationType): string {
  return Object.values(location).join('-');
}
