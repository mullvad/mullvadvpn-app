import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { type GeographicalLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { type ListItemProps } from '../../../../../lib/components/list-item';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { getLocationListItemMapProps } from '../../utils';
import { Location } from '../location-list-item';
import { GeographicalLocationTrailingActions } from './components';
import {
  GeographicalLocationProvider,
  useGeographicalLocationContext,
} from './GeographicalLocationContext';

export type GeographicalLocationProps = Pick<ListItemProps, 'level' | 'position'> & {
  location: GeographicalLocation;
  root?: boolean;
  disabled?: boolean;
  onSelect: (location: GeographicalLocation) => void;
  expanded?: boolean;
};

function GeographicalLocationImpl({
  location,
  level,
  disabled: disabledProp,
  root,
  position,
  onSelect,
  ...props
}: GeographicalLocationProps) {
  const { loading } = useGeographicalLocationContext();
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
        <GeographicalLocation
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
    <Location selected={location.selected} root={root}>
      <Location.Accordion expanded={expanded} onExpandedChange={setExpanded} disabled={disabled}>
        <Location.Accordion.Header
          ref={location.selected ? selectedLocationRef : null}
          level={level}
          position={position}>
          <Location.Accordion.Header.ItemTrigger
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
            <Location.Accordion.Header.Item>
              <Location.Accordion.Header.Item.Title>
                {location.label}
              </Location.Accordion.Header.Item.Title>
            </Location.Accordion.Header.Item>
          </Location.Accordion.Header.ItemTrigger>
          <GeographicalLocationTrailingActions location={location} />
        </Location.Accordion.Header>
        <Location.Accordion.Content>
          {showChildren ? renderChildren() : null}
        </Location.Accordion.Content>
      </Location.Accordion>
    </Location>
  );
}

export function GeographicalLocation({ ...props }: GeographicalLocationProps) {
  return (
    <GeographicalLocationProvider>
      <GeographicalLocationImpl {...props} />
    </GeographicalLocationProvider>
  );
}
