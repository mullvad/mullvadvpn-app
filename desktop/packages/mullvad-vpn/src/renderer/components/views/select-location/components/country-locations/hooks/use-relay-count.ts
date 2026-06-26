import { useLocationListsContext } from '../../location-lists/LocationListsContext';
import { useRelayLocationsFilteredByLocationType } from './use-relay-locations-filtered-by-location-type';

export function useRelayCount() {
  const { countryLocations } = useLocationListsContext();
  const relayLocationsFiltered = useRelayLocationsFilteredByLocationType();

  const visibleRelays = countryLocations.reduce(
    (countryAcc, country) =>
      countryAcc + country.cities.reduce((cityAcc, city) => cityAcc + city.relays.length, 0),
    0,
  );

  const matchedRelays = relayLocationsFiltered.matches.length;
  const discardedRelays = relayLocationsFiltered.discards.length;

  const totalRelays = matchedRelays + discardedRelays;

  return { visibleRelays, totalRelays };
}
