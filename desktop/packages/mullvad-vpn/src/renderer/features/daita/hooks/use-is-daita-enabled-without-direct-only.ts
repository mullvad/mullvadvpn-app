import { useDaitaDirectOnly } from './use-daita-direct-only';
import { useDaitaEnabled } from './use-daita-enabled';

export function useIsDaitaEnabledWithoutDirectOnly() {
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();

  const isDaitaWithoutDirectOnly = daitaEnabled && !daitaDirectOnly;

  return isDaitaWithoutDirectOnly;
}
