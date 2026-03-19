import type { AnyLocation } from '../../../../features/locations/types';

export function getLocationListItemMapProps(location: AnyLocation, level?: number) {
  const key = Object.values(location.details).join('-');
  const nextLevel = level !== undefined ? level + 1 : undefined;
  return {
    key,
    nextLevel,
  };
}
