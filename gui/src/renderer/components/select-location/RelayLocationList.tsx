import React from 'react';

import { RelayLocation, relayLocationComponents } from '../../../shared/daemon-rpc-types';
import * as Cell from '../cell';
import LocationRow from './LocationRow';
import {
  getLocationChildren,
  LocationSelection,
  LocationSpecification,
  RelayList,
} from './select-location-types';

interface CommonProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: LocationSelection<never>) => void;
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
        <RelayLocation key={getLocationKey(country.location)} source={country} {...props} />
      ))}
    </Cell.Group>
  );
}

interface RelayLocationProps extends CommonProps {
  source: LocationSpecification;
}

function RelayLocation(props: RelayLocationProps) {
  const children = getLocationChildren(props.source);

  return (
    <LocationRow {...props}>
      {children.map((child) => (
        <RelayLocation key={getLocationKey(child.location)} {...props} source={child} />
      ))}
    </LocationRow>
  );
}

function getLocationKey(location: RelayLocation): string {
  return relayLocationComponents(location).join('-');
}
