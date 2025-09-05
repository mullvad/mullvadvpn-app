import { useApplication } from './use-application';

export function useHasApplicationWarning() {
  const application = useApplication();

  const hasApplicationWarning = application.warning !== undefined;

  return hasApplicationWarning;
}
