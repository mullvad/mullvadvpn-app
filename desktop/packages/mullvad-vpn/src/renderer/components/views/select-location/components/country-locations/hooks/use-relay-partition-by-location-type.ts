import { LocationType } from '../../../../../../features/locations/types';
import { useSelector } from '../../../../../../redux/store';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useRelayPartitionByLocationType() {
  const { locationType } = useSelectLocationViewContext();
  const relayPartitions = useSelector((state) => state.settings.relayPartitions);

  switch (locationType) {
    case LocationType.entry:
      return relayPartitions.partitions.entry;
    case LocationType.exit:
      return relayPartitions.partitions.exit;
    default:
      return locationType satisfies never;
  }
}
