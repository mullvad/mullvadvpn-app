import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { type AnyLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { LocationListItem } from '../location-list-item';
import {
  AnyLocationListItemProvider,
  useAnyLocationListItemContext,
} from './AnyLocationListItemContext';
import { CustomListTrailingActions, GeographicalLocationTrailingActions } from './components';

export type AnyLocationListItemProps = React.PropsWithChildren<{
  location: AnyLocation;
  root?: boolean;
  rootLocation?: 'customList' | 'geographical';
  level: ListItemProps['level'];
  position?: ListItemProps['position'];
  disabled?: boolean;
  onSelect: (location: AnyLocation) => void;
  onExpandedChange: (value: boolean) => void;
  expanded: boolean;
}>;

function AnyLocationListItemImpl({
  level,
  position,
  disabled,
  onSelect,
  root,
  children,
  expanded,
  onExpandedChange,
}: Omit<AnyLocationListItemProps, 'location' | 'rootLocation'>) {
  const { location } = useAnyLocationListItemContext();
  const { selectedLocationRef } = useScrollPositionContext();

  const showEmptySubtitle = location.type === 'customList' && !children;

  const handleClick = React.useCallback(() => {
    onSelect(location);
  }, [location, onSelect]);

  const selectedRef = location.selected ? selectedLocationRef : undefined;
  return (
    <LocationListItem selected={location.selected} root={root}>
      <LocationListItem.Accordion
        expanded={expanded}
        onExpandedChange={onExpandedChange}
        disabled={location.disabled || disabled}>
        <LocationListItem.Accordion.Header ref={selectedRef} level={level} position={position}>
          <LocationListItem.Accordion.Header.Trigger
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
            <LocationListItem.Accordion.Header.Item>
              <FlexColumn>
                <LocationListItem.Accordion.Header.Item.Title>
                  {location.label}
                </LocationListItem.Accordion.Header.Item.Title>
                {showEmptySubtitle && (
                  <FootnoteMiniSemiBold color="whiteAlpha60">
                    {
                      // TRANSLATORS: Label for custom lists that don't have any locations added to them yet.
                      messages.pgettext('select-location-view', 'Empty')
                    }
                  </FootnoteMiniSemiBold>
                )}
              </FlexColumn>
            </LocationListItem.Accordion.Header.Item>
          </LocationListItem.Accordion.Header.Trigger>
          {location.type === 'customList' ? (
            <CustomListTrailingActions customList={location} />
          ) : (
            <GeographicalLocationTrailingActions location={location} />
          )}
        </LocationListItem.Accordion.Header>
        <LocationListItem.Accordion.Content>{children}</LocationListItem.Accordion.Content>
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
