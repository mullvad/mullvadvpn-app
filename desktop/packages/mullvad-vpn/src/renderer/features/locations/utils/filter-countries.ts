import type {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';

export function filterCountries(
  countries: IRelayLocationCountryRedux[],
  filter: (relay: IRelayLocationRelayRedux) => boolean,
): IRelayLocationCountryRedux[] {
  let anyCountryChanged = false;

  const filteredCountries = countries
    .map((country) => {
      let anyCityChanged = false;
      const cities = country.cities
        .map((city) => {
          const relays = city.relays.filter(filter);
          if (relays.length === city.relays.length) {
            return city;
          }

          anyCityChanged = true;
          return { ...city, relays };
        })
        .filter((city) => city.relays.length > 0);

      if (!anyCityChanged && cities.length === country.cities.length) {
        return country;
      }

      anyCountryChanged = true;
      return { ...country, cities };
    })
    .filter((country) => country.cities.length > 0);

  if (!anyCountryChanged && filteredCountries.length === countries.length) {
    return countries;
  }

  return filteredCountries;
}
