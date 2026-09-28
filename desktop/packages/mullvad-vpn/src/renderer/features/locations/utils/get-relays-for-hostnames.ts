import { type IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { findRelay } from './find-location';

export function getRelaysForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const relays = hostnames
    .map((hostname) => findRelay(hostname, locations))
    .filter((relay) => relay !== undefined)
    .filter((relay, i, existingRelays) => {
      // Filter out duplicates
      const firstIndex = existingRelays.findIndex(
        (existingRelays) => existingRelays?.hostname === relay?.hostname,
      );

      return firstIndex === i;
    });

  return relays;
}
