import { useIdentity, useUserInterfaceConnectedToDaemon } from '../../../../redux/hooks';

// The detailed settings are available as soon as there is a config to connect with.
export const useShowSubSettings = () => {
  const { identity } = useIdentity();
  const connectedToDaemon = useUserInterfaceConnectedToDaemon();
  return identity !== undefined && connectedToDaemon;
};
