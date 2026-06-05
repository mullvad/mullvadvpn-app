import { RelaySelectorRelayDiscard } from '../../../../shared/relay-selector-rpc-types';
import { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getRelayPartitionDiscardsFilter(
  relayDiscards: RelaySelectorRelayDiscard[],
  multihop: boolean,
) {
  return (relay: IRelayLocationRelayRedux) =>
    relayDiscards.some((discardedRelay) => {
      if (discardedRelay.relay.hostname === relay.hostname) {
        // Check if a discarded relay should be included in the filtered result
        const result = Object.entries(discardedRelay.why).every(([key, value]) => {
          // Always include inactive servers
          if (key === 'inactive') {
            return true;
          }

          // If multihop is enabled we want to be able to present the conflicting server in the relay list
          if (key === 'conflictWithOtherHop' && multihop) {
            return true;
          }

          // If a discarded relay was only originally filtered out by the Relay selector only the special
          // cases handled above, then it should be included in the result, however if it was filtered out
          // for another reason it should be discarded.
          return !value;
        });

        return result;
      }

      return false;
    });
}
