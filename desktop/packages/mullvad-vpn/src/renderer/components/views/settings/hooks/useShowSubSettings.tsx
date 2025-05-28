import { useAccountStatus, useUserInterfaceConnectedToDaemon } from '../../../../redux/hooks';

export const useShowSubSettings = () => {
  const { status } = useAccountStatus();
  const connectedToDaemon = useUserInterfaceConnectedToDaemon();
  return status.type === 'ok' && connectedToDaemon;
};
