import { useIsUsingMagicMultihop, useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useShowExitFilterChip() {
  const { isolatedItem } = useSelectLocationViewContext();
  const { multihop } = useMultihop();
  const isUsingMagicMultihop = useIsUsingMagicMultihop();

  const showExitFilterChip =
    isolatedItem === 'exit' && (multihop === 'always' || isUsingMagicMultihop);

  return showExitFilterChip;
}
