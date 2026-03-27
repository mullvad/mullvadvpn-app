import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useRelayCount() {
  const { countryLocations } = useSelectLocationViewContext();
  const { relayLocations } = useRelayLocations();

  const visibleRelays = countryLocations.reduce(
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
