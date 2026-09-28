import { LiftedConstraint, RelayLocation } from '../../../../../../../shared/daemon-rpc-types';
import { useGetIsCustomListInHostnameLocations } from './use-get-is-custom-list-in-hostname-locations';
import { useGetIsGeographicalLocationInHostnameLocations } from './use-get-is-geographical-location-in-hostname-locations';

export function useIsLocationInHostnameLocations(
  location: LiftedConstraint<RelayLocation>,
  hostnames: string[],
) {
  const getIsGeographicalLocationInHostnameLocations =
    useGetIsGeographicalLocationInHostnameLocations(hostnames);
  const getIsCustomListInHostnameLocations = useGetIsCustomListInHostnameLocations(hostnames);

  // TODO: Is this a good assumption for how to treat 'any'?
  if (location === 'any') {
    return hostnames.length > 0;
  }

  if ('customList' in location) {
    return getIsCustomListInHostnameLocations(location);
  }

  return getIsGeographicalLocationInHostnameLocations(location);
}
