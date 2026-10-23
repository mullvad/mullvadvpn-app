import { type IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { findCityForRelay } from './find-location';

export function getCitiesForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const cities = hostnames
    .map((hostname) => findCityForRelay(hostname, locations))
    .filter((city) => city !== undefined)
    .filter((city, i, existingCities) => {
      // Filter out duplicates
      const firstIndex = existingCities.findIndex(
        (existingCity) => existingCity?.code === city.code,
      );

      return firstIndex === i;
    });

  return cities;
}
