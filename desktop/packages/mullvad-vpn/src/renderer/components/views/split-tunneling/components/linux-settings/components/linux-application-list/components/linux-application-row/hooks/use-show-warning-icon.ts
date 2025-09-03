import { useHasApplicationWarning } from './use-has-application-warning';

export function useShowWarningIcon() {
  const showWarningIcon = useHasApplicationWarning();

  return showWarningIcon;
}
