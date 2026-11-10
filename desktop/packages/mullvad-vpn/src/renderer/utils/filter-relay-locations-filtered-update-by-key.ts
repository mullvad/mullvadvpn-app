import {
  type RelayLocationsFilterContext,
  type RelayLocationsFiltered,
} from '../redux/settings/reducers';

type FilterRelaySelectorQueryOrResultByKeyProps = {
  context: RelayLocationsFilterContext;
  ignoreCache: boolean;
  key: string;
  relayLocationsFiltered: RelayLocationsFiltered;
};
export function filterRelayLocationsFilteredUpdateByKey({
  context,
  ignoreCache,
  key,
  relayLocationsFiltered,
}: FilterRelaySelectorQueryOrResultByKeyProps) {
  if (ignoreCache || !(context in relayLocationsFiltered)) {
    return true;
  }

  const relayLocationsFilteredContextKey = relayLocationsFiltered[context].key;

  return key !== relayLocationsFilteredContextKey;
}
