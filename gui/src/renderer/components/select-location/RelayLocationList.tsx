import React from 'react';

import { RelayLocation } from '../../../shared/daemon-rpc-types';
import * as Cell from '../cell';
import LocationRow from './LocationRow';
import { getLocationChildren, LocationSpecification, RelayList } from './select-location-types';

interface CommonProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  allowAddToCustomList: boolean;
  onSelect: (value: RelayLocation) => void;
  onExpand: (location: RelayLocation) => void;
  onCollapse: (location: RelayLocation) => void;
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

export default function RelayLocationList({ source, ...props }: RelayLocationsProps) {
  return (
    <Cell.Group noMarginBottom>
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

interface RelayLocationProps extends CommonProps {
  source: LocationSpecification;
  level: number;
}

function RelayLocation(props: RelayLocationProps) {
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

function getLocationKey(location: RelayLocation): string {
  return Object.values(location).join('-');
}
