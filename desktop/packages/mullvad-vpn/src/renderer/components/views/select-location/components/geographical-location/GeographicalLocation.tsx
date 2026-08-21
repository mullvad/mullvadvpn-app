import { useCallback, useEffect, useState } from 'react';

import { useRecents } from '../../../../../features/locations/hooks';
import { type GeographicalLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { type ListItemProps } from '../../../../../lib/components/list-item';
import { useLocationAriaLabel } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { getLocationListItemMapProps } from '../../utils';
import { Location } from '../location-list-item';
import { useLocationSlideContext } from '../location-slide/LocationSlideContext';
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
  const { searchTerm } = useSelectLocationViewContext();
  const { loading } = useGeographicalLocationContext();
  const [expanded, setExpanded] = useState(location.expanded);
  const [lastSearchTerm, setLastSearchTerm] = useState(searchTerm);
  const locationChildren = getLocationChildren(location);
  const { selectedLocationRef } = useLocationSlideContext();
  const { hasRecents } = useRecents();

  const ariaLabel = useLocationAriaLabel(location.label);

  // If search term changes, reset expanded state.
  useEffect(() => {
    if (searchTerm !== lastSearchTerm) {
      setLastSearchTerm(searchTerm);
      setExpanded(location.expanded);
    }
  }, [searchTerm, lastSearchTerm, location.expanded]);

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

  // Only scroll to the selected location when the recents feature is disabled
  const shouldScrollToLocation = location.selected && !hasRecents;
  const refToScrollTo = shouldScrollToLocation ? selectedLocationRef : null;

  return (
    <Location selected={location.selected} root={root}>
      <Location.Accordion expanded={expanded} onExpandedChange={setExpanded} disabled={disabled}>
        <Location.Accordion.Header ref={refToScrollTo} level={level} position={position}>
          <Location.Accordion.Header.ItemTrigger onClick={handleClick} aria-label={ariaLabel}>
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
