import { useApplication } from './use-application';

export function useDisabled() {
  const application = useApplication();

  const disabled = application.warning === 'launches-elsewhere';

  return disabled;
}
