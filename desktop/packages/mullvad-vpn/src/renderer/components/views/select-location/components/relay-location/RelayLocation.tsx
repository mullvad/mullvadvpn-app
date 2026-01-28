import React from 'react';

import { type RelayLocation as RelayLocationType } from '../../../../../../shared/daemon-rpc-types';
import { getLocationChildren, LocationSpecification } from '../../select-location-types';
import LocationRow from '../location-row/LocationRow';

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

interface RelayLocationProps extends CommonProps {
  source: LocationSpecification;
  level: number;
}

export function RelayLocation(props: RelayLocationProps) {
  const children = getLocationChildren(props.source);

  return (
    <LocationRow {...props}>
      {children.map((child) => (
        <RelayLocation
          key={getLocationKey(child.location)}
          {...props}
          source={child}
          level={props.level + 1}
        />
      ))}
    </LocationRow>
  );
}

function getLocationKey(location: RelayLocationType): string {
  return Object.values(location).join('-');
}
