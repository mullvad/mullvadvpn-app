import { useVersionSuggestedUpgrade } from '../../../../redux/hooks';

export function useHasUpgrade() {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  const hasUpgrade = suggestedUpgrade !== undefined;

  return hasUpgrade;
}
