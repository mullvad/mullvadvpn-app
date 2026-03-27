import { useSelector } from '../../../redux/store';
import { useObfuscation } from '../../anti-censorship/hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../daita/hooks';
import { useMultihop } from '../../multihop/hooks';
import { useIpVersion } from '../../tunnel/hooks';
import { type LocationType } from '../types';
import { filterLocations } from '../utils';
import { useOwnership, useProviders } from '.';

export function useFilterCountryLocations(locationType: LocationType) {
  const locations = useSelector((state) => state.settings.relayLocations);
  const { activeOwnership } = useOwnership();
  const { activeProviders } = useProviders();
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { obfuscation } = useObfuscation();
  const { multihop } = useMultihop();
  const { ipVersion } = useIpVersion();

  return filterLocations({
    locations,
    ownership: activeOwnership,
    providers: activeProviders,
    daita: daitaEnabled,
    directOnly: daitaDirectOnly,
    locationType,
    multihop,
    obfuscation,
    ipVersion,
  });
}
