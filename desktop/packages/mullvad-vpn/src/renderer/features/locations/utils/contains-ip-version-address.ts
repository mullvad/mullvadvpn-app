import type { IpVersion, LiftedConstraint } from '../../../../shared/daemon-rpc-types';
import { IpAddress, IPv4Address, IPv6Address } from '../../../lib/ip';

export function containsIpVersionAddress(
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
