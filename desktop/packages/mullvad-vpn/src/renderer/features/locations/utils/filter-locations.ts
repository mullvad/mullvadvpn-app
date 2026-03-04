import type {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';

export function filterLocations(
  locations: Array<IRelayLocationCountryRedux>,
  filter: (relay: IRelayLocationRelayRedux) => boolean,
): Array<IRelayLocationCountryRedux> {
  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({ ...city, relays: city.relays.filter(filter) }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}
