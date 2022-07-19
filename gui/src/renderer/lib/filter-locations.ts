import { Ownership } from '../../shared/daemon-rpc-types';
import { IRelayLocationRedux } from '../redux/settings/reducers';

export default function filterLocations(
  locations: IRelayLocationRedux[],
  providers: string[],
  ownership: Ownership,
): IRelayLocationRedux[] {
  const locationsFilteredByOwnership = filterLocationsByOwnership(locations, ownership);
  const locationsFilteredByProvider = filterLocationsByProvider(
    locationsFilteredByOwnership,
    providers,
  );

  return locationsFilteredByProvider;
}

function filterLocationsByOwnership(
  locations: IRelayLocationRedux[],
  ownership: Ownership,
): IRelayLocationRedux[] {
  if (ownership === Ownership.any) {
    return locations;
  }

  const expectOwned = ownership === Ownership.mullvadOwned;
  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({
          ...city,
          relays: city.relays.filter((relay) => relay.owned === expectOwned),
        }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}

function filterLocationsByProvider(
  locations: IRelayLocationRedux[],
  providers: string[],
): IRelayLocationRedux[] {
  if (providers.length === 0) {
    return locations;
  }

  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({
          ...city,
          relays: city.relays.filter((relay) => providers.includes(relay.provider)),
        }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}
