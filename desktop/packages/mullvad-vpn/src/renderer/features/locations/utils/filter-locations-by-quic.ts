import type { IpVersion, LiftedConstraint } from '../../../../shared/daemon-rpc-types';
import type {
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { containsIpVersionAddress } from './contains-ip-version-address';
import { filterLocations } from './filter-locations';
import { isQuicFilterActive } from './is-quic-filter-active';

export function filterLocationsByQuic(
  locations: IRelayLocationCountryRedux[],
  quic: boolean,
  locationType: LocationType,
  multihop: boolean,
  ipVersion: LiftedConstraint<IpVersion>,
): IRelayLocationCountryRedux[] {
  const quickOnRelay = (relay: IRelayLocationRelayRedux) =>
    relay.quic !== undefined && containsIpVersionAddress(relay.quic.addrIn, ipVersion);
  return isQuicFilterActive(quic, locationType, multihop)
    ? filterLocations(locations, quickOnRelay)
    : locations;
}
