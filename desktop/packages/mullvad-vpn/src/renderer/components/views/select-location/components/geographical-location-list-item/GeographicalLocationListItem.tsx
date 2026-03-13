import { getLocationChildren } from '../../../../../features/locations/utils';
import { getLocationListItemMapProps } from '../../utils';
import { AnyLocationListItem, type AnyLocationListItemProps } from '../any-location-list-item';
import {
  GeographicalLocationListItemProvider,
  useGeographicalLocationListItemContext,
} from './GeographicalLocationListItemContext';

export type GeographicalLocationListItemProps = AnyLocationListItemProps;

function GeographicalLocationListItemImpl({
  location,
  level,
  disabled,
  root,
  ...props
}: GeographicalLocationListItemProps) {
  const { loading } = useGeographicalLocationListItemContext();

  const children = getLocationChildren(location);
  return (
    <AnyLocationListItem
      location={location}
      root={root}
      rootLocation="geographical"
      level={level}
      disabled={disabled || loading}
      {...props}>
      {children.map((child) => {
        const { key, nextLevel } = getLocationListItemMapProps(child, level);
        return (
          <GeographicalLocationListItem
            key={key}
            location={child}
            rootLocation="geographical"
            level={nextLevel}
            disabled={disabled || loading}
            {...props}
          />
        );
      })}
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
