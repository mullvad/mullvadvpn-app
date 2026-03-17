import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { type GeographicalLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { type ListItemProps } from '../../../../../lib/components/list-item';
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

  useEffect(() => {
    setExpanded(location.expanded);
  }, [location.expanded]);

  const disabled = disabledProp || location.disabled || loading;
  const showChildren = locationChildren.length > 0 && expanded;

  const handleSelect = useCallback(() => {
    onSelect(location);
  }, [location, onSelect]);

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
        <LocationListItem.Header level={level} position={position}>
          <LocationListItem.HeaderTrigger
            onClick={handleSelect}
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
          <GeographicalLocationTrailingActions location={location} />
        </LocationListItem.Header>
        <LocationListItem.AccordionContent>
          {showChildren ? renderChildren() : null}
        </LocationListItem.AccordionContent>
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
