import { getLocationChildrenByType } from '../../select-location-types';
import { AnyLocationListItem, type AnyLocationListItemProps } from '../any-location-list-item';

export type GeographicalLocationListItemProps = AnyLocationListItemProps;

export function GeographicalLocationListItem({
  location,
  level,
  ...props
}: GeographicalLocationListItemProps) {
  const children = getLocationChildrenByType(location);
  return (
    <AnyLocationListItem location={location} level={level} {...props}>
      {children.map((child) => {
        return (
          <GeographicalLocationListItem
            key={Object.values(child.details).join('-')}
            location={child}
            level={level !== undefined ? level + 1 : undefined}
            {...props}
          />
        );
      })}
    </AnyLocationListItem>
  );
}
