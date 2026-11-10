import { useSelectLocationViewContext } from '../../../../../components/views/select-location/SelectLocationViewContext';
import { useMultihop } from '../../../../multihop/hooks';
import { DisabledReason, type GeographicalLocation } from '../../../types';

export function useShowSetAsEntryMenuOption(location: GeographicalLocation) {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();

  return (
    multihop === 'always' &&
    locationType === 'entry' &&
    location.disabledReason !== DisabledReason.entry
  );
}
