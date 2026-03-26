import type { GeographicalLocation } from '../../../../../../features/locations/types';
import { useSelector } from '../../../../../../redux/store';

export function useLocationBreadcrumbs(geographicalLocation: GeographicalLocation) {
  const locations = useSelector((state) => state.settings.relayLocations);
  if (geographicalLocation.type === 'city') {
    const country = locations.find(
      (location) => location.code === geographicalLocation.details.country,
    );

    return country ? [country.name] : [];
  } else if (geographicalLocation.type === 'relay') {
    const city = locations
      .flatMap((location) => location.cities)
      .find((city) => city.code === geographicalLocation.details.city);
    if (city) {
      const country = locations.find((location) =>
        location.cities.some((c) => c.code === city.code),
      );

      return country ? [city.name, country.name] : [city.name];
    }
  }

  return [];
}
