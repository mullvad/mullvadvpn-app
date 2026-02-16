import { getLocationChildrenByType } from '../../select-location-types';
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
  ...props
}: GeographicalLocationListItemProps) {
  const { loading } = useGeographicalLocationListItemContext();

  const children = getLocationChildrenByType(location);
  return (
    <AnyLocationListItem
      location={location}
      rootLocation="geographical"
      level={level}
      disabled={disabled || loading}
      {...props}>
      {children.map((child) => {
        return (
          <GeographicalLocationListItem
            key={Object.values(child.details).join('-')}
            location={child}
            rootLocation="geographical"
            level={level !== undefined ? level + 1 : undefined}
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
