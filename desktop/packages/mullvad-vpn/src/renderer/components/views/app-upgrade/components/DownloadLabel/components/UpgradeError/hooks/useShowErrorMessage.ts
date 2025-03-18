import { useShowStartingInstallerFailed } from './useShowStartingInstallerFailed';

export const useShowErrorMessage = () => {
  const showStartingInstallerFailed = useShowStartingInstallerFailed();

  const showErrorMessage = !showStartingInstallerFailed;

  return showErrorMessage;
};
