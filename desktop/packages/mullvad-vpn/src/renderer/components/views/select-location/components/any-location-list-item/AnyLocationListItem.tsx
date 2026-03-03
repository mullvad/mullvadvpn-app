import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import type { RelayLocation as DaemonRelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { type AnyLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { LocationListItem } from '../../../../location-list-item';
import { useScrollPositionContext } from '../../ScrollPositionContext';
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

  const childLocations = getLocationChildren(location);
  const hasChildren = childLocations.length > 0;
  const showEmptySubtitle = location.type === 'customList' && !hasChildren;

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
              <FlexColumn>
                <LocationListItem.HeaderTitle>{location.label}</LocationListItem.HeaderTitle>
                {showEmptySubtitle && (
                  <FootnoteMiniSemiBold color="whiteAlpha60">
                    {
                      // TRANSLATORS: Label for custom lists that don't have any locations added to them yet.
                      messages.pgettext('select-location-view', 'Empty')
                    }
                  </FootnoteMiniSemiBold>
                )}
              </FlexColumn>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>
          {location.type === 'customList' ? (
            <CustomListTrailingActions customList={location} />
          ) : (
            <GeographicalLocationTrailingActions location={location} />
          )}
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
