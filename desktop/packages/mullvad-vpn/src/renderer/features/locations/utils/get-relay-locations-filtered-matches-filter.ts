import { RelaySelectorRelayMatch } from '../../../../shared/relay-selector-rpc-types';
import { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getRelayLocationsFilteredMatchesFilter(relayMatches: RelaySelectorRelayMatch[]) {
  return (relay: IRelayLocationRelayRedux) =>
    relayMatches.some(({ relay: { hostname } }) => relay.hostname === hostname);
}
