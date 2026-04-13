import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { type GeographicalLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { type ListItemProps } from '../../../../../lib/components/list-item';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { getLocationListItemMapProps } from '../../utils';
import { LocationListItem } from '../location-list-item';
import { GeographicalLocationTrailingActions } from './components';
import {
  GeographicalLocationListItemProvider,
  useGeographicalLocationListItemContext,
} from './GeographicalLocationListItemContext';

export type GeographicalLocationListItemProps = Pick<ListItemProps, 'level' | 'position'> & {
  location: GeographicalLocation;
  root?: boolean;
  disabled?: boolean;
  onSelect: (location: GeographicalLocation) => void;
  expanded?: boolean;
};

function GeographicalLocationListItemImpl({
  location,
  level,
  disabled: disabledProp,
  root,
  position,
  onSelect,
  ...props
}: GeographicalLocationListItemProps) {
  const { loading } = useGeographicalLocationListItemContext();
  const [expanded, setExpanded] = useState(location.expanded);
  const locationChildren = getLocationChildren(location);
  const { selectedLocationRef } = useScrollPositionContext();

  useEffect(() => {
    setExpanded(location.expanded);
  }, [location.expanded]);

  const disabled = disabledProp || location.disabled || loading;
  const showChildren = locationChildren.length > 0 && expanded;

  const handleClick = useCallback(() => {
    onSelect(location);
  }, [location, onSelect]);

  const handleSelect = useCallback(
    (location: GeographicalLocation) => {
      onSelect(location);
    },
    [onSelect],
  );

  const renderChildren = () => {
    return locationChildren.map((locationChild) => {
      const { key, nextLevel } = getLocationListItemMapProps(locationChild, level);
      return (
        <GeographicalLocationListItem
          key={key}
          location={locationChild}
          level={nextLevel}
          disabled={disabled}
          onSelect={handleSelect}
          {...props}
        />
      );
    });
  };

  return (
    <LocationListItem selected={location.selected} root={root}>
      <LocationListItem.Accordion
        expanded={expanded}
        onExpandedChange={setExpanded}
        disabled={disabled}>
        <LocationListItem.Accordion.Header
          ref={location.selected ? selectedLocationRef : null}
          level={level}
          position={position}>
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
              <LocationListItem.Accordion.Header.Item.Title>
                {location.label}
              </LocationListItem.Accordion.Header.Item.Title>
            </LocationListItem.Accordion.Header.Item>
          </LocationListItem.Accordion.Header.Trigger>
          <GeographicalLocationTrailingActions location={location} />
        </LocationListItem.Accordion.Header>
        <LocationListItem.Accordion.Content>
          {showChildren ? renderChildren() : null}
        </LocationListItem.Accordion.Content>
      </LocationListItem.Accordion>
    </LocationListItem>
  );
}

export function GeographicalLocationListItem({ ...props }: GeographicalLocationListItemProps) {
  return (
    <GeographicalLocationListItemProvider>
      <GeographicalLocationListItemImpl {...props} />
    </GeographicalLocationListItemProvider>
  );
}
