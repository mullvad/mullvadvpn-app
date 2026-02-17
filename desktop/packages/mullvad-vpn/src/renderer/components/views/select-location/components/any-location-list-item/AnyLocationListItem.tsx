import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import type { RelayLocation as DaemonRelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { LocationListItem } from '../../../../location-list-item';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { type AnyLocation, getLocationChildrenByType } from '../../select-location-types';
import {
  AnyLocationListItemProvider,
  useAnyLocationListItemContext,
} from './AnyLocationListItemContext';
import { CustomListTrailingActions, GeographicalLocationTrailingActions } from './components';

export type AnyLocationListItemProps = React.PropsWithChildren<{
  location: AnyLocation;
  rootLocation?: 'customList' | 'geographical';
  level: ListItemProps['level'];
  position?: ListItemProps['position'];
  disabled?: boolean;
  onSelect: (value: DaemonRelayLocation) => void;
}>;

function AnyLocationListItemImpl({
  level,
  position,
  disabled,
  onSelect,
  children,
}: Omit<AnyLocationListItemProps, 'location' | 'rootLocation'>) {
  const { location, expanded, setExpanded } = useAnyLocationListItemContext();
  const { selectedLocationRef } = useScrollPositionContext();

  const childLocations = getLocationChildrenByType(location);
  const hasChildren = childLocations.length > 0;

  const handleClick = useCallback(() => {
    if (!location.selected) {
      onSelect(location.details);
    }
  }, [location.details, location.selected, onSelect]);

  const selectedRef = location.selected ? selectedLocationRef : undefined;
  return (
    <LocationListItem selected={location.selected}>
      <LocationListItem.Accordion
        expanded={expanded && hasChildren}
        onExpandedChange={setExpanded}
        disabled={location.disabled || disabled}>
        <LocationListItem.Header ref={selectedRef} level={level} position={position}>
          <LocationListItem.HeaderTrigger
            onClick={handleClick}
            aria-label={sprintf(messages.pgettext('accessibility', 'Connect to %(location)s'), {
              location: location.label,
            })}>
            <LocationListItem.HeaderItem>
              <LocationListItem.Title>{location.label}</LocationListItem.Title>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>
          {location.type !== 'customList' && (
            <GeographicalLocationTrailingActions location={location} />
          )}
          {location.type === 'customList' && <CustomListTrailingActions customList={location} />}
        </LocationListItem.Header>

        <LocationListItem.AccordionContent>{children}</LocationListItem.AccordionContent>
      </LocationListItem.Accordion>
    </LocationListItem>
  );
}

export function AnyLocationListItem({
  location,
  rootLocation,
  ...props
}: AnyLocationListItemProps) {
  return (
    <AnyLocationListItemProvider location={location} rootLocation={rootLocation}>
      <AnyLocationListItemImpl {...props} />
    </AnyLocationListItemProvider>
  );
}
