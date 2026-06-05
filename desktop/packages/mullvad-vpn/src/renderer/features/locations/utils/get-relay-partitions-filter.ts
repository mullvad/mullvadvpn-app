import {
  type IRelayLocationRelayRedux,
  type RelayPartitions,
  type RelayPartitionsContext,
} from '../../../redux/settings/reducers';
import { getRelayPartitionDiscardsFilter } from './get-relay-partition-discards-filter';
import { getRelayPartitionMatchesFilter } from './get-relay-partition-matches-filter';

export function getRelayPartitionsFilter(
  relayPartitions: RelayPartitions,
  context: RelayPartitionsContext,
  multihop: boolean,
): (relay: IRelayLocationRelayRedux) => boolean {
  return (relay) => {
    const relayPartition = relayPartitions.partitions[context];
    const filters = [
      getRelayPartitionMatchesFilter(relayPartition.matches),
      getRelayPartitionDiscardsFilter(relayPartition.discards, multihop),
    ];

    return filters.some((filter) => filter(relay));
  };
}
