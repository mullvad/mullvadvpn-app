import { useDaitaEnabled } from './use-daita-enabled';

export function useIsDaitaEnabledWithoutDirectOnly() {
  const { daitaEnabled } = useDaitaEnabled();
  // TODO: Should we account for `multhop-mode: when-needed` here?
  return daitaEnabled;
}
