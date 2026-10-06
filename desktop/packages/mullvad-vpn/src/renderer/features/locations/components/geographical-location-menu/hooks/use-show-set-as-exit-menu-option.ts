import { useSelectLocationViewContext } from '../../../../../components/views/select-location/SelectLocationViewContext';
import { useMultihop } from '../../../../multihop/hooks';
import { DisabledReason, type GeographicalLocation } from '../../../types';

export function useShowSetAsExitMenuOption(location: GeographicalLocation) {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();

  return (
    multihop !== 'never' &&
    locationType === 'entry' &&
    location.disabledReason !== DisabledReason.exit
  );
}
