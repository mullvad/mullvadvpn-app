import { ConnectionBlocked, InstallerReady, StartUpgrade } from './components';
import { useShowConnectionBlocked, useShowInstallerReady } from './hooks';

export function InitialFooter() {
  const showConnectionBlocked = useShowConnectionBlocked();
  const showInstallerReady = useShowInstallerReady();

  if (showInstallerReady) {
    return <InstallerReady />;
  }

  if (showConnectionBlocked) {
    return <ConnectionBlocked />;
  }

  return <StartUpgrade />;
}
