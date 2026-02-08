import { getLocationChildrenByType } from '../../select-location-types';
import LocationRow, { type LocationRowProps } from '../location-row/LocationRow';

export type RelayLocationProps = LocationRowProps;

export function RelayLocation({ location, level, ...props }: RelayLocationProps) {
  const children = getLocationChildrenByType(location);
  return (
    <LocationRow location={location} level={level} {...props}>
      {children.map((child) => {
        return (
          <RelayLocation
            key={Object.values(child.details).join('-')}
            location={child}
            level={level !== undefined ? level + 1 : undefined}
            {...props}
          />
        );
      })}
    </LocationRow>
  );
}
