import { useSelectLocationViewContext } from '../../../../../components/views/select-location/SelectLocationViewContext';
import { useMultihop } from '../../../../multihop/hooks';
import { DisabledReason, type GeographicalLocation, LocationType } from '../../../types';

export function useShowSetAsEntryMenuOption(location: GeographicalLocation) {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();

  return (
    multihop !== 'never' &&
    // TODO: Replace this logic to actually lookup in the filteredRelays in Redux
    // whether this location can be selected as entry
    locationType === LocationType.exit &&
    location.disabledReason !== DisabledReason.entry
  );
}
