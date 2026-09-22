import type {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';

export function filterCountries(
  countries: IRelayLocationCountryRedux[],
  filter: (relay: IRelayLocationRelayRedux) => boolean,
): IRelayLocationCountryRedux[] {
  return countries
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({ ...city, relays: city.relays.filter(filter) }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}
