import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';
import { useRelayPartitionByLocationType } from './use-relay-partition-by-location-type';

export function useRelayCount() {
  const { countryLocations } = useSelectLocationViewContext();
  const relayPartition = useRelayPartitionByLocationType();

  const visibleRelays = countryLocations.reduce(
    (countryAcc, country) =>
      countryAcc + country.cities.reduce((cityAcc, city) => cityAcc + city.relays.length, 0),
    0,
  );

  const matchedRelays = relayPartition.matches.length;
  const discardedRelays = relayPartition.discards.length;

  const totalRelays = matchedRelays + discardedRelays;

  return { visibleRelays, totalRelays };
}
