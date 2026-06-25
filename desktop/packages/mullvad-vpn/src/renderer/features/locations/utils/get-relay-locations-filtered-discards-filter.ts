import { MultihopMode } from '../../../../shared/daemon-rpc-types';
import { RelaySelectorRelayDiscard } from '../../../../shared/relay-selector-rpc-types';
import { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getRelayLocationsFilteredDiscardsFilter(
  relayDiscards: RelaySelectorRelayDiscard[],
  multihop: MultihopMode,
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

          // If multihop is enabled we want to be able to present the conflicting server in the
          // relay list.
          if (key === 'conflictWithOtherHop' && multihop !== 'never') {
            return true;
          }

          // If a relay was discarded by the Relay selector, and the reasons for discarding the
          // relay wasn't already handled by the special cases handled above, then it should be
          // filtered out by us as well.
          //
          // If the `value` is true it means that the relay was discarded by the Relay selector,
          // and as such it will be filtered out here too.
          return !value;
        });

        return result;
      }

      return false;
    });
}
