import React from 'react';

import { type RelayLocation as RelayLocationType } from '../../../../../../shared/daemon-rpc-types';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { getLocationChildren, type LocationSpecification } from '../../select-location-types';
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
  level: ListItemProps['level'];
}

export function RelayLocation({ source, level, ...props }: RelayLocationProps) {
  const children = getLocationChildren(source);
  return (
    <LocationRow source={source} level={level} {...props}>
      {children.map((child) => {
        return (
          <RelayLocation
            key={getLocationKey(child.location)}
            source={child}
            level={level !== undefined ? level + 1 : undefined}
            {...props}
          />
        );
      })}
    </LocationRow>
  );
}

function getLocationKey(location: RelayLocationType): string {
  return Object.values(location).join('-');
}
