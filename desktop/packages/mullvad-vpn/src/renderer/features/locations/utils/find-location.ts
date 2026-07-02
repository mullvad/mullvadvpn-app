import type { ICustomList } from '../../../../shared/daemon-rpc-types';
import type {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';

export function findCountry(
  code: string,
  locations: IRelayLocationCountryRedux[],
): IRelayLocationCountryRedux | undefined {
  return locations.find((location) => location.code === code);
}

export function findCity(
  code: string,
  locations: IRelayLocationCountryRedux[],
): IRelayLocationCityRedux | undefined {
  return locations.flatMap((country) => country.cities).find((city) => city.code === code);
}

export function findRelay(
  hostname: string,
  locations: IRelayLocationCountryRedux[],
): IRelayLocationRelayRedux | undefined {
  return locations
    .flatMap((country) => country.cities)
    .flatMap((city) => city.relays)
    .find((relay) => relay.hostname === hostname);
}

export function findCustomList(
  id: string,
  customLists: Array<ICustomList>,
): ICustomList | undefined {
  return customLists.find((list) => list.id === id);
}
