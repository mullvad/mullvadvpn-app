import { IRelayLocationCountryRedux } from '../../../../redux/settings/reducers';
import { removeDuplicates } from './remove-duplicates';

// Returns all available providers in the provided relay list.
export function providersFromRelays(relays: IRelayLocationCountryRedux[]) {
  const providers = relays.flatMap((country) =>
    country.cities.flatMap((city) => city.relays.map((relay) => relay.provider)),
  );
  return removeDuplicates(providers).sort((a, b) => a.localeCompare(b));
}
