import type { IpVersion, LiftedConstraint } from '../../../../shared/daemon-rpc-types';
import { IpAddress, IPv4Address, IPv6Address } from '../../../lib/ip';
import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { isQuicFilterActive } from './is-quic-filter-active';

export function getQuicFilter(
  quic: boolean,
  locationType: LocationType,
  multihop: boolean,
  ipVersion: LiftedConstraint<IpVersion>,
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  const filterActive = isQuicFilterActive(quic, locationType, multihop);
  return filterActive
    ? (relay: IRelayLocationRelayRedux) =>
        relay.quic !== undefined && containsIpVersionAddress(relay.quic.addrIn, ipVersion)
    : undefined;
}

function containsIpVersionAddress(
  addresses: string[],
  version: LiftedConstraint<IpVersion>,
): boolean {
  if (version === 'any') {
    return addresses.length > 0;
  }
  return addresses.some((strAddr) => {
    try {
      const addr = IpAddress.fromString(strAddr);
      return (
        (addr instanceof IPv4Address && version === 'ipv4') ||
        (addr instanceof IPv6Address && version === 'ipv6')
      );
    } catch {
      return false;
    }
  });
}
