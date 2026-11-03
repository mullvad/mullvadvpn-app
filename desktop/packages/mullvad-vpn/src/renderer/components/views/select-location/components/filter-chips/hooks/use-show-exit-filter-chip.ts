import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useShowExitFilterChip() {
  const { isolatedItem } = useSelectLocationViewContext();
  const { multihop } = useMultihop();

  const showExitFilterChip = multihop === 'always' && isolatedItem === 'exit';

  return showExitFilterChip;
}
