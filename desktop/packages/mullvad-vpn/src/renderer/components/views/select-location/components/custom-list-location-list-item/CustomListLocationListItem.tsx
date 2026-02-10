import { getLocationChildrenByType } from '../../select-location-types';
import { AnyLocationListItem, type AnyLocationListItemProps } from '../any-location-list-item';

export type CustomListLocationListItemProps = AnyLocationListItemProps;

export function CustomListLocationListItem({
  location,
  level,
  ...props
}: CustomListLocationListItemProps) {
  const children = getLocationChildrenByType(location);
  return (
    <AnyLocationListItem location={location} level={level} {...props}>
      {children.map((child) => {
        return (
          <CustomListLocationListItem
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
