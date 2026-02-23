import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import type { RelayLocation as DaemonRelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { spacings } from '../../../../../lib/foundations';
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

const StyledAccordionContent = styled(LocationListItem.AccordionContent)`
  // Last accordion content for a location should have extra spacing at the bottom
  &:not(:has(&)):last-child {
    margin-bottom: ${spacings.small};
  }
`;

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
            aria-label={sprintf(
              // TRANSLATORS: Accessibility label for a button that connects to a location.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
              messages.pgettext('accessibility', 'Connect to %(location)s'),
              {
                location: location.label,
              },
            )}>
            <LocationListItem.HeaderItem>
              <LocationListItem.HeaderTitle>{location.label}</LocationListItem.HeaderTitle>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>
          {location.type !== 'customList' && (
            <GeographicalLocationTrailingActions location={location} />
          )}
          {location.type === 'customList' && <CustomListTrailingActions customList={location} />}
        </LocationListItem.Header>

        <StyledAccordionContent>{children}</StyledAccordionContent>
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
