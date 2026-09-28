import { findCountryForRelay } from '../../../../../../features/locations/utils';
import { type IRelayLocationCountryRedux } from '../../../../../../redux/settings/reducers';

export function getCountriesForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const countries = hostnames
    .map((hostname) => findCountryForRelay(hostname, locations))
    .filter((relay) => relay !== undefined);

  return countries; // TODO: remove duplicates
}
