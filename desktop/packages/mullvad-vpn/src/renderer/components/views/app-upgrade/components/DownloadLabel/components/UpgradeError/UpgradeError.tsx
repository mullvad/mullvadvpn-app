import { ErrorMessage, StartingInstallerFailed } from './components';
import { useShowErrorMessage, useShowStartingInstallerFailed } from './hooks';

export function UpgradeError() {
  const showErrorMessage = useShowErrorMessage();
  const showStartingInstallerFailed = useShowStartingInstallerFailed();

  return (
    <>
      {showErrorMessage ? <ErrorMessage /> : null}
      {showStartingInstallerFailed ? <StartingInstallerFailed /> : null}
    </>
  );
}
