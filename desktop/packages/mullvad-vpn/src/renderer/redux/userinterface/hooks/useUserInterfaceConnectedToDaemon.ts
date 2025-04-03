import { useSelector } from '../../store';

export const useUserInterfaceConnectedToDaemon = () => {
  return {
    connectedToDaemon: useSelector((state) => state.userInterface.connectedToDaemon),
  };
};
