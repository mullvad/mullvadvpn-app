import { RelaySelectorRelayMatch } from '../../../../shared/relay-selector-rpc-types';
import { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getRelayPartitionMatchesFilter(relayMatches: RelaySelectorRelayMatch[]) {
  return (relay: IRelayLocationRelayRedux) =>
    relayMatches.some(({ hostname }) => relay.hostname === hostname);
}
