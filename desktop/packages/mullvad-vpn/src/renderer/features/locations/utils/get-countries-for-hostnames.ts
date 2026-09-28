import { type IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { findCountryForRelay } from './find-location';

export function getCountriesForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const countries = hostnames
    .map((hostname) => findCountryForRelay(hostname, locations))
    .filter((relay) => relay !== undefined);

  return countries; // TODO: remove duplicates
}
