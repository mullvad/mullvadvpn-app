import { useDaitaDirectOnly, useDaitaEnabled } from '../../../../features/daita/hooks';
import { useMultihop } from '../../../../features/multihop/hooks';
import { daitaFilterActive } from '../../../../lib/filter-locations';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useDaitaFilterActive() {
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { locationType } = useSelectLocationViewContext();
  const { multihop } = useMultihop();

  return daitaFilterActive(daitaEnabled, daitaDirectOnly, locationType, multihop);
}
