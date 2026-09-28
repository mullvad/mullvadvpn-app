import { findRelay } from '../../../../../../features/locations/utils';
import { type IRelayLocationCountryRedux } from '../../../../../../redux/settings/reducers';

export function getRelaysForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const relays = hostnames
    .map((hostname) => findRelay(hostname, locations))
    .filter((relay) => relay !== undefined);

  return relays; // TODO: remove duplicates
}
