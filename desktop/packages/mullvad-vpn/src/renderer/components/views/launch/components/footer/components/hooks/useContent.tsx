import { useSelector } from '../../../../../../../redux/store';
import { DefaultLaunchFooter, MacOsPermissionFooter, RestartDaemonFooter } from '../../..';

export const useContent = () => {
  const platform = window.env.platform;
  const daemonAllowed = useSelector((state) => state.userInterface.daemonAllowed);
  if (platform === 'darwin' && daemonAllowed === false) return <MacOsPermissionFooter />;
  if (platform === 'win32') return <RestartDaemonFooter />;
  return <DefaultLaunchFooter />;
};
