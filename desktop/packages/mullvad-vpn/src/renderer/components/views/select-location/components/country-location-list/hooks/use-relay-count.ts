import { useRelayLocations } from '../../../../../../features/location/hooks';
import { useLocationsContext } from '../../../LocationsContext';

export function useRelayCount() {
  const { searchedLocations } = useLocationsContext();
  const { relayLocations } = useRelayLocations();

  const visibleRelays = searchedLocations.reduce(
    (countryAcc, country) =>
      countryAcc + country.cities.reduce((cityAcc, city) => cityAcc + city.relays.length, 0),
    0,
  );

  const totalRelays = relayLocations.reduce(
    (countryAcc, country) =>
      countryAcc + country.cities.reduce((cityAcc, city) => cityAcc + city.relays.length, 0),
    0,
  );

  return { visibleRelays, totalRelays };
}
