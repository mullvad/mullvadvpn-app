import { useState } from 'react';

import { getLocationChildren } from '../../../../../features/locations/utils';
import { getLocationListItemMapProps } from '../../utils';
import { AnyLocationListItem, type AnyLocationListItemProps } from '../any-location-list-item';
import {
  GeographicalLocationListItemProvider,
  useGeographicalLocationListItemContext,
} from './GeographicalLocationListItemContext';

export type GeographicalLocationListItemProps = Omit<
  AnyLocationListItemProps,
  'children' | 'expanded' | 'onExpandedChange'
>;

function GeographicalLocationListItemImpl({
  location,
  level,
  disabled: disabledProp,
  root,
  rootLocation,
  position,
  onSelect,
  ...props
}: GeographicalLocationListItemProps) {
  const { loading } = useGeographicalLocationListItemContext();
  const [locationExpanded, setLocationExpanded] = useState(location.expanded);
  const locationChildren = getLocationChildren(location);
  const expanded = location.expanded || locationExpanded;
  const disabled = disabledProp || loading;
  const showChildren = locationChildren.length > 0 && expanded;

  const renderChildren = () => {
    return locationChildren.map((locationChild) => {
      const { key, nextLevel } = getLocationListItemMapProps(locationChild, level);
      return (
        <GeographicalLocationListItem
          key={key}
          location={locationChild}
          rootLocation={rootLocation}
          level={nextLevel}
          disabled={disabled}
          onSelect={onSelect}
          {...props}
        />
      );
    });
  };

  return (
    <AnyLocationListItem
      location={location}
      root={root}
      rootLocation={rootLocation}
      level={level}
      disabled={disabled}
      expanded={expanded}
      onExpandedChange={setLocationExpanded}
      position={position}
      onSelect={onSelect}
      {...props}>
      {showChildren ? renderChildren() : null}
    </AnyLocationListItem>
  );
}

export function GeographicalLocationListItem({ ...props }: GeographicalLocationListItemProps) {
  return (
    <GeographicalLocationListItemProvider>
      <GeographicalLocationListItemImpl {...props} />
    </GeographicalLocationListItemProvider>
  );
}
