import { useSettingsRelayLocationsFiltered } from '../../../redux/hooks';
import { RelayLocationsFilterContext } from '../../../redux/settings/reducers';
import { useSelector } from '../../../redux/store';
import { useMultihop } from '../../multihop/hooks';
import { LocationType } from '../types';
import { filterCountriesByRelayLocationsFiltered } from '../utils';

export function useFilterCountryLocations(locationType: LocationType) {
  const locations = useSelector((state) => state.settings.relayLocations);
  const { relayLocationsFiltered } = useSettingsRelayLocationsFiltered();
  const { multihop } = useMultihop();
  const context: RelayLocationsFilterContext =
    locationType === LocationType.exit ? 'exit' : 'entry';

  return filterCountriesByRelayLocationsFiltered(
    locations,
    relayLocationsFiltered,
    context,
    multihop,
  );
}
