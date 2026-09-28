import { type IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { findCountryForRelay } from './find-location';

export function getCountriesForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const countries = hostnames
    .map((hostname) => findCountryForRelay(hostname, locations))
    .filter((countries) => countries !== undefined)
    .filter((country, i, existingCountries) => {
      // Filter out duplicates
      const firstIndex = existingCountries.findIndex(
        (existingCountry) => existingCountry?.code === country.code,
      );

      return firstIndex !== i;
    });

  return countries;
}
